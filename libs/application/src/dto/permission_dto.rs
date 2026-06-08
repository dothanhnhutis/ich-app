use domain::entities::Permission;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PermissionItem {
    pub id: String,
    pub code: String,
    pub description: String,
}

impl From<Permission> for PermissionItem {
    fn from(p: Permission) -> Self {
        Self {
            id: p.id.to_string(),
            code: p.code,
            description: p.description,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PermissionGroup {
    pub prefix: String,
    pub permissions: Vec<PermissionItem>,
}

#[derive(Debug, Serialize)]
pub struct PermissionsResponse {
    pub groups: Vec<PermissionGroup>,
}

impl PermissionsResponse {
    /// Gom `Vec<Permission>` (ĐÃ sort theo code) thành nhóm theo prefix (phần trước dấu `_` đầu tiên).
    /// Input cùng prefix nằm liền kề nên gom tuyến tính bằng `last_mut()`, giữ thứ tự xác định.
    pub fn group_by_prefix(perms: Vec<Permission>) -> Self {
        let mut groups: Vec<PermissionGroup> = Vec::new();
        for p in perms {
            let prefix = p.code.split('_').next().unwrap_or("").to_string();
            let item = PermissionItem::from(p);
            match groups.last_mut() {
                Some(g) if g.prefix == prefix => g.permissions.push(item),
                _ => groups.push(PermissionGroup {
                    prefix,
                    permissions: vec![item],
                }),
            }
        }
        Self { groups }
    }
}
