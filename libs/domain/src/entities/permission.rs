use uuid::Uuid;

/// Một permission trong hệ thống (catalog). `code` dạng `PREFIX_ACTION` (vd USER_CREATE).
#[derive(Debug, Clone)]
pub struct Permission {
    pub id: Uuid,
    pub code: String,
    pub description: String,
}
