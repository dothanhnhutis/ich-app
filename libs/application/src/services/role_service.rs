use std::str::FromStr;

use uuid::Uuid;
use validator::Validate;

use crate::dto::pagination::Paginated;
use crate::dto::permission_dto::PermissionsResponse;
use crate::dto::role_dto::{
    CreateRoleRequest, ListRolesQuery, RoleResponse, UpdateRoleRequest,
};
use crate::errors::AppError;
use domain::entities::{
    NewRole, RoleFilter, RoleSort, RoleSortField, RoleStatus, RoleUpdate, SortDir,
};
use crate::ports::RoleRepository;

const DEFAULT_PAGE_SIZE: u32 = 20;
const MAX_PAGE_SIZE: u32 = 100;

/// Parse chuỗi trạng thái → RoleStatus, lỗi → Validation (message tiếng Việt rõ).
fn parse_status(s: &str) -> Result<RoleStatus, AppError> {
    RoleStatus::from_str(s).map_err(|_| AppError::Validation("Trạng thái không hợp lệ".into()))
}

/// Chuẩn hoá filter chuỗi: trim, rỗng → None (không lọc).
fn norm(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// Parse chuỗi sort `field:dir,field:dir` → Vec<RoleSort> (thiếu hướng → asc); lỗi → 400.
fn parse_sort(raw: &str) -> Result<Vec<RoleSort>, AppError> {
    let mut out = Vec::new();
    for token in raw.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        let (f, d) = match token.split_once(':') {
            Some((f, d)) => (f.trim(), d.trim()),
            None => (token, "asc"),
        };
        let field = RoleSortField::from_str(&f.to_lowercase())
            .map_err(|_| AppError::Validation(format!("Trường sắp xếp không hợp lệ: {f}")))?;
        let dir = SortDir::from_str(&d.to_lowercase())
            .map_err(|_| AppError::Validation(format!("Hướng sắp xếp không hợp lệ: {d}")))?;
        out.push(RoleSort { field, dir });
    }
    Ok(out)
}

pub struct RoleService<RR>
where
    RR: RoleRepository,
{
    role_repo: RR,
}

impl<RR> RoleService<RR>
where
    RR: RoleRepository,
{
    pub fn new(role_repo: RR) -> Self {
        Self { role_repo }
    }

    /// Tạo vai trò mới + gán permission (cần ROLE_CREATE — kiểm ở middleware).
    pub async fn create_role(&self, req: CreateRoleRequest) -> Result<RoleResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        // Loại trùng permission_id → tránh đụng PK kép của role_permissions.
        let mut ids = req.permission_ids;
        ids.sort();
        ids.dedup();

        let role = self
            .role_repo
            .create_with_permissions(
                NewRole {
                    name: req.name.trim().to_string(),
                    description: req.description,
                },
                &ids,
            )
            .await?;
        Ok(RoleResponse::from(role))
    }

    /// Danh sách vai trò (lọc + phân trang).
    pub async fn list_roles(
        &self,
        q: ListRolesQuery,
    ) -> Result<Paginated<RoleResponse>, AppError> {
        let page = q.page.unwrap_or(1).max(1);
        let page_size = q.page_size.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);

        let status = match q.status.as_deref() {
            Some(s) => Some(parse_status(s)?),
            None => None,
        };

        let sort = match q.sort.as_deref() {
            Some(s) => parse_sort(s)?,
            None => Vec::new(),
        };

        let filter = RoleFilter {
            name: norm(q.name),
            description: norm(q.description),
            status,
            sort,
            limit: page_size as i64,
            offset: ((page - 1) * page_size) as i64,
        };

        let (roles, total) = self.role_repo.list(filter).await?;

        let total_pages = if total == 0 {
            0
        } else {
            ((total as u64).div_ceil(page_size as u64)) as u32
        };

        Ok(Paginated {
            items: roles.into_iter().map(RoleResponse::from).collect(),
            page,
            page_size,
            total_items: total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        })
    }

    /// Cập nhật vai trò (name/description/status). Chặn role hệ thống (can_update=false) → 403.
    pub async fn update_role(
        &self,
        id: Uuid,
        req: UpdateRoleRequest,
    ) -> Result<RoleResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let role = self
            .role_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Vai trò không tồn tại".into()))?;
        if !role.can_update {
            return Err(AppError::Forbidden(
                "Vai trò hệ thống không thể chỉnh sửa".into(),
            ));
        }

        let status = match req.status.as_deref() {
            Some(s) => Some(parse_status(s)?),
            None => None,
        };

        let changes = RoleUpdate {
            name: req.name.map(|n| n.trim().to_string()),
            description: req.description,
            status,
            // Dedupe để tránh đụng PK kép khi INSERT lại role_permissions.
            permission_ids: req.permission_ids.map(|mut v| {
                v.sort();
                v.dedup();
                v
            }),
        };

        let updated = self
            .role_repo
            .update(id, changes)
            .await?
            .ok_or_else(|| AppError::NotFound("Vai trò không tồn tại".into()))?;
        Ok(RoleResponse::from(updated))
    }

    /// Xoá mềm vai trò. Chặn role hệ thống (can_delete=false) → 403.
    pub async fn delete_role(&self, id: Uuid) -> Result<(), AppError> {
        let role = self
            .role_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Vai trò không tồn tại".into()))?;
        if !role.can_delete {
            return Err(AppError::Forbidden("Vai trò hệ thống không thể xoá".into()));
        }

        self.role_repo.soft_delete(id).await?;
        Ok(())
    }

    /// Danh sách permission của một role, nhóm theo prefix (cần ROLE_VIEW — kiểm ở middleware).
    pub async fn role_permissions_grouped(
        &self,
        id: Uuid,
    ) -> Result<PermissionsResponse, AppError> {
        // 404 nếu role không tồn tại / đã xoá mềm (phân biệt với "role có 0 quyền").
        self.role_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Vai trò không tồn tại".into()))?;
        let perms = self.role_repo.find_permissions_for_role(id).await?;
        Ok(PermissionsResponse::group_by_prefix(perms))
    }
}
