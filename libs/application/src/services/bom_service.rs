use std::str::FromStr;

use uuid::Uuid;
use validator::Validate;

use crate::dto::bom_dto::{
    AddBomLineRequest, BomLineResponse, BomResponse, BomWithLinesResponse, CreateBomRequest,
    ListBomsQuery, UpdateBomLineRequest, UpdateBomRequest,
};
use crate::dto::pagination::Paginated;
use crate::errors::AppError;
use domain::entities::{
    BomFilter, BomLineType, BomLineUpdate, BomSort, BomSortField, BomStatus, BomType, BomUpdate,
    NewBom, NewBomLine, QtyBasis, SortDir,
};
use domain::repositories::{BomRepository, ItemRepository};

const DEFAULT_PAGE_SIZE: u32 = 20;
const MAX_PAGE_SIZE: u32 = 100;

fn norm(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn parse_bom_type(s: &str) -> Result<BomType, AppError> {
    BomType::from_str(s.trim()).map_err(|_| AppError::Validation("Loại BOM không hợp lệ".into()))
}

fn parse_bom_status(s: &str) -> Result<BomStatus, AppError> {
    BomStatus::from_str(s.trim())
        .map_err(|_| AppError::Validation("Trạng thái BOM không hợp lệ".into()))
}

fn parse_qty_basis(s: &str) -> Result<QtyBasis, AppError> {
    QtyBasis::from_str(s.trim())
        .map_err(|_| AppError::Validation("Cơ sở định lượng không hợp lệ".into()))
}

fn parse_line_type(s: &str) -> Result<BomLineType, AppError> {
    BomLineType::from_str(s.trim())
        .map_err(|_| AppError::Validation("Loại dòng BOM không hợp lệ".into()))
}

fn parse_sort(raw: &str) -> Result<Vec<BomSort>, AppError> {
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
        let field = BomSortField::from_str(&f.to_lowercase())
            .map_err(|_| AppError::Validation(format!("Trường sắp xếp không hợp lệ: {f}")))?;
        let dir = SortDir::from_str(&d.to_lowercase())
            .map_err(|_| AppError::Validation(format!("Hướng sắp xếp không hợp lệ: {d}")))?;
        out.push(BomSort { field, dir });
    }
    Ok(out)
}

pub struct BomService<BR, IR>
where
    BR: BomRepository,
    IR: ItemRepository,
{
    bom_repo: BR,
    item_repo: IR,
}

impl<BR, IR> BomService<BR, IR>
where
    BR: BomRepository,
    IR: ItemRepository,
{
    pub fn new(bom_repo: BR, item_repo: IR) -> Self {
        Self { bom_repo, item_repo }
    }

    /// Item đầu ra phải tồn tại & là bán thành phẩm / thành phẩm (mới được có BOM).
    async fn ensure_output_item(&self, output_item_id: Uuid) -> Result<(), AppError> {
        let item = self
            .item_repo
            .find_by_id(output_item_id)
            .await?
            .ok_or_else(|| AppError::Validation("Item đầu ra không tồn tại".into()))?;
        if !item.item_type.can_have_bom() {
            return Err(AppError::Validation(
                "BOM chỉ áp dụng cho bán thành phẩm hoặc thành phẩm".into(),
            ));
        }
        Ok(())
    }

    /// Thành phần phải là item tồn tại (mọi loại đều được làm thành phần).
    async fn ensure_component(&self, component_item_id: Uuid) -> Result<(), AppError> {
        self.item_repo
            .find_by_id(component_item_id)
            .await?
            .ok_or_else(|| AppError::Validation("Thành phần (item) không tồn tại".into()))?;
        Ok(())
    }

    /// Chống tự tham chiếu & chu trình BOM (đệ quy qua repo).
    async fn guard_cycle(&self, component_id: Uuid, output_id: Uuid) -> Result<(), AppError> {
        if component_id == output_id {
            return Err(AppError::Validation(
                "Thành phần không thể trùng với item đầu ra của BOM".into(),
            ));
        }
        if self
            .bom_repo
            .would_create_cycle(component_id, output_id)
            .await?
        {
            return Err(AppError::Validation(
                "Thêm thành phần này sẽ tạo chu trình BOM".into(),
            ));
        }
        Ok(())
    }

    /// Tạo BOM kèm dòng (transaction) (cần BOM_CREATE — kiểm ở middleware).
    pub async fn create_bom(
        &self,
        req: CreateBomRequest,
    ) -> Result<BomWithLinesResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let bom_type = parse_bom_type(&req.bom_type)?;
        let status = match req.status.as_deref() {
            Some(s) => parse_bom_status(s)?,
            None => BomStatus::Draft,
        };
        let qty_basis = match req.qty_basis.as_deref() {
            Some(s) => parse_qty_basis(s)?,
            None => QtyBasis::Absolute,
        };

        self.ensure_output_item(req.output_item_id).await?;

        let mut new_lines: Vec<NewBomLine> = Vec::with_capacity(req.lines.len());
        for (idx, line) in req.lines.iter().enumerate() {
            let line_type = match line.line_type.as_deref() {
                Some(s) => parse_line_type(s)?,
                None => BomLineType::Item,
            };
            self.ensure_component(line.component_item_id).await?;
            self.guard_cycle(line.component_item_id, req.output_item_id)
                .await?;
            new_lines.push(NewBomLine {
                component_item_id: line.component_item_id,
                line_no: line.line_no.unwrap_or((idx + 1) as i32),
                line_type,
                quantity: line.quantity,
                input_uom: norm(line.input_uom.clone()),
                input_qty: line.input_qty,
                scrap_pct: line.scrap_pct.unwrap_or(0.0),
                is_gift: line.is_gift,
                notes: norm(line.notes.clone()),
            });
        }

        let new_bom = NewBom {
            output_item_id: req.output_item_id,
            bom_type,
            code: req.code.trim().to_string(),
            name: req.name.trim().to_string(),
            version_no: req.version_no.unwrap_or(1),
            status,
            is_default: req.is_default,
            qty_basis,
            output_qty: req.output_qty,
            output_uom: req.output_uom.trim().to_string(),
            effective_from: req.effective_from,
            effective_to: req.effective_to,
            notes: norm(req.notes),
        };

        let (bom, lines) = self.bom_repo.create_with_lines(new_bom, &new_lines).await?;
        Ok(BomWithLinesResponse::from((bom, lines)))
    }

    /// Danh sách BOM (lọc + phân trang + sắp xếp) (cần BOM_VIEW).
    pub async fn list_boms(
        &self,
        q: ListBomsQuery,
    ) -> Result<Paginated<BomResponse>, AppError> {
        let page = q.page.unwrap_or(1).max(1);
        let page_size = q.page_size.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
        let sort = match q.sort.as_deref() {
            Some(s) => parse_sort(s)?,
            None => Vec::new(),
        };

        let filter = BomFilter {
            output_item_id: q.output_item_id,
            bom_type: norm(q.bom_type),
            status: norm(q.status),
            code: norm(q.code),
            sort,
            limit: page_size as i64,
            offset: ((page - 1) * page_size) as i64,
        };

        let (boms, total) = self.bom_repo.list(filter).await?;

        let total_pages = if total == 0 {
            0
        } else {
            ((total as u64).div_ceil(page_size as u64)) as u32
        };

        Ok(Paginated {
            items: boms.into_iter().map(BomResponse::from).collect(),
            page,
            page_size,
            total_items: total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        })
    }

    /// Chi tiết BOM kèm dòng (cần BOM_VIEW).
    pub async fn get_bom(&self, id: Uuid) -> Result<BomWithLinesResponse, AppError> {
        let with_lines = self
            .bom_repo
            .find_with_lines(id)
            .await?
            .ok_or_else(|| AppError::NotFound("BOM không tồn tại".into()))?;
        Ok(BomWithLinesResponse::from(with_lines))
    }

    /// Cập nhật BOM header (cần BOM_UPDATE).
    pub async fn update_bom(
        &self,
        id: Uuid,
        req: UpdateBomRequest,
    ) -> Result<BomResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let status = match req.status.as_deref() {
            Some(s) => Some(parse_bom_status(s)?),
            None => None,
        };
        let qty_basis = match req.qty_basis.as_deref() {
            Some(s) => Some(parse_qty_basis(s)?),
            None => None,
        };

        let changes = BomUpdate {
            code: req.code.map(|c| c.trim().to_string()),
            name: req.name.map(|n| n.trim().to_string()),
            version_no: req.version_no,
            status,
            is_default: req.is_default,
            qty_basis,
            output_qty: req.output_qty,
            output_uom: req.output_uom.map(|u| u.trim().to_string()),
            effective_from: req.effective_from,
            effective_to: req.effective_to,
            notes: norm(req.notes),
        };

        let updated = self
            .bom_repo
            .update(id, changes)
            .await?
            .ok_or_else(|| AppError::NotFound("BOM không tồn tại".into()))?;
        Ok(BomResponse::from(updated))
    }

    /// Xoá mềm BOM + cascade dòng (cần BOM_DELETE).
    pub async fn delete_bom(&self, id: Uuid) -> Result<(), AppError> {
        self.bom_repo.soft_delete(id).await?;
        Ok(())
    }

    /// Thêm một dòng vào BOM (cần BOM_UPDATE).
    pub async fn add_line(
        &self,
        bom_id: Uuid,
        req: AddBomLineRequest,
    ) -> Result<BomLineResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let bom = self
            .bom_repo
            .find_by_id(bom_id)
            .await?
            .ok_or_else(|| AppError::NotFound("BOM không tồn tại".into()))?;

        let line_type = match req.line_type.as_deref() {
            Some(s) => parse_line_type(s)?,
            None => BomLineType::Item,
        };
        self.ensure_component(req.component_item_id).await?;
        self.guard_cycle(req.component_item_id, bom.output_item_id)
            .await?;

        // line_no: dùng giá trị gửi lên, hoặc tự tăng theo dòng lớn nhất hiện có.
        let line_no = match req.line_no {
            Some(n) => n,
            None => {
                let existing = self.bom_repo.list_lines(bom_id).await?;
                existing.iter().map(|l| l.line_no).max().unwrap_or(0) + 1
            }
        };

        let line = self
            .bom_repo
            .add_line(
                bom_id,
                NewBomLine {
                    component_item_id: req.component_item_id,
                    line_no,
                    line_type,
                    quantity: req.quantity,
                    input_uom: norm(req.input_uom),
                    input_qty: req.input_qty,
                    scrap_pct: req.scrap_pct.unwrap_or(0.0),
                    is_gift: req.is_gift,
                    notes: norm(req.notes),
                },
            )
            .await?;
        Ok(BomLineResponse::from(line))
    }

    /// Cập nhật một dòng BOM (cần BOM_UPDATE). `component_item_id` bị khoá → không re-check cycle.
    pub async fn update_line(
        &self,
        bom_id: Uuid,
        line_id: Uuid,
        req: UpdateBomLineRequest,
    ) -> Result<BomLineResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let line_type = match req.line_type.as_deref() {
            Some(s) => Some(parse_line_type(s)?),
            None => None,
        };

        let changes = BomLineUpdate {
            line_no: req.line_no,
            line_type,
            quantity: req.quantity,
            input_uom: norm(req.input_uom),
            input_qty: req.input_qty,
            scrap_pct: req.scrap_pct,
            is_gift: req.is_gift,
            notes: norm(req.notes),
        };

        let updated = self
            .bom_repo
            .update_line(bom_id, line_id, changes)
            .await?
            .ok_or_else(|| AppError::NotFound("Dòng BOM không tồn tại".into()))?;
        Ok(BomLineResponse::from(updated))
    }

    /// Xoá mềm một dòng BOM (cần BOM_UPDATE).
    pub async fn delete_line(&self, bom_id: Uuid, line_id: Uuid) -> Result<(), AppError> {
        self.bom_repo.soft_delete_line(bom_id, line_id).await?;
        Ok(())
    }
}
