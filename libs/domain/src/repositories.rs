use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::entities::{
    Bin, BinFilter, BinUpdate, Location, LocationFilter, LocationUpdate, NewBin, NewLocation,
    NewPasswordToken, NewRole, NewSession, NewUser, NewZone, PasswordToken, PasswordTokenType,
    Permission, Role, RoleFilter, RoleUpdate, Session, User, UserFilter, UserUpdate, Zone,
    ZoneFilter, ZoneUpdate,
};
use crate::errors::DomainError;

pub trait UserRepository: Send + Sync {
    fn find_by_email(
        &self,
        email: &str,
    ) -> impl Future<Output = Result<Option<User>, DomainError>> + Send;

    fn find_by_id(
        &self,
        id: uuid::Uuid,
    ) -> impl Future<Output = Result<Option<User>, DomainError>> + Send;

    /// Danh sách user (lọc + phân trang + sắp xếp); trả về (items, tổng số khớp).
    fn list(
        &self,
        filter: UserFilter,
    ) -> impl Future<Output = Result<(Vec<User>, i64), DomainError>> + Send;

    /// Tạo user mới (PENDING_PASSWORD) và gán role trong MỘT transaction.
    fn create_with_roles(
        &self,
        new_user: NewUser,
        role_ids: &[Uuid],
    ) -> impl Future<Output = Result<User, DomainError>> + Send;

    /// Cập nhật username/status; None nếu user không tồn tại / đã xoá mềm.
    fn update(
        &self,
        id: Uuid,
        changes: UserUpdate,
    ) -> impl Future<Output = Result<Option<User>, DomainError>> + Send;

    /// Xoá mềm (set deleted_at). NotFound nếu user không tồn tại / đã xoá.
    fn soft_delete(&self, id: Uuid) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// INIT: đặt username + mật khẩu + kích hoạt (ACTIVE) + đánh dấu token đã dùng, atomic.
    fn activate_account(
        &self,
        user_id: Uuid,
        username: &str,
        password_hash: &str,
        token_id: Uuid,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// RESET: đặt mật khẩu mới (+ password_changed_at) + đánh dấu token đã dùng, atomic.
    /// Không đụng status/username (user đã ACTIVE).
    fn reset_password(
        &self,
        user_id: Uuid,
        password_hash: &str,
        token_id: Uuid,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;
}

pub trait RoleRepository: Send + Sync {
    /// Mã permission của một user (JOIN user_roles→role_permissions→permissions),
    /// chỉ tính role đang ACTIVE và chưa xoá mềm.
    fn find_permission_codes_for_user(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<String>, DomainError>> + Send;

    /// Toàn bộ permission trong hệ thống (catalog), sắp xếp theo code.
    fn find_all_permissions(
        &self,
    ) -> impl Future<Output = Result<Vec<Permission>, DomainError>> + Send;

    /// Tạo vai trò mới + gán permission trong MỘT transaction.
    fn create_with_permissions(
        &self,
        new_role: NewRole,
        permission_ids: &[Uuid],
    ) -> impl Future<Output = Result<Role, DomainError>> + Send;

    /// Tìm vai trò theo id (chỉ role chưa xoá mềm).
    fn find_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<Role>, DomainError>> + Send;

    /// Permission của một role (JOIN role_permissions→permissions), sắp xếp theo code.
    fn find_permissions_for_role(
        &self,
        role_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Permission>, DomainError>> + Send;

    /// Các role được gán cho một user (JOIN user_roles→roles), chỉ role chưa xoá mềm.
    fn find_roles_for_user(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Role>, DomainError>> + Send;

    /// Danh sách vai trò (lọc + phân trang); trả về (items, tổng số khớp).
    fn list(
        &self,
        filter: RoleFilter,
    ) -> impl Future<Output = Result<(Vec<Role>, i64), DomainError>> + Send;

    /// Cập nhật một phần; None nếu role không tồn tại / đã xoá mềm.
    fn update(
        &self,
        id: Uuid,
        changes: RoleUpdate,
    ) -> impl Future<Output = Result<Option<Role>, DomainError>> + Send;

    /// Xoá mềm (set deleted_at). NotFound nếu role không tồn tại / đã xoá.
    fn soft_delete(&self, id: Uuid) -> impl Future<Output = Result<(), DomainError>> + Send;
}

pub trait LocationRepository: Send + Sync {
    /// Tạo kho mới.
    fn create(
        &self,
        new_location: NewLocation,
    ) -> impl Future<Output = Result<Location, DomainError>> + Send;

    /// Tìm kho theo id (chỉ kho chưa xoá mềm).
    fn find_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<Location>, DomainError>> + Send;

    /// Danh sách kho (lọc + phân trang + sắp xếp); trả về (items, tổng số khớp).
    fn list(
        &self,
        filter: LocationFilter,
    ) -> impl Future<Output = Result<(Vec<Location>, i64), DomainError>> + Send;

    /// Cập nhật một phần; None nếu kho không tồn tại / đã xoá mềm.
    fn update(
        &self,
        id: Uuid,
        changes: LocationUpdate,
    ) -> impl Future<Output = Result<Option<Location>, DomainError>> + Send;

    /// Xoá mềm (set deleted_at). NotFound nếu kho không tồn tại / đã xoá.
    fn soft_delete(&self, id: Uuid) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Còn khu vực (zone) nào CHƯA xoá mềm thuộc kho này không (để chặn xoá kho).
    fn has_active_zones(
        &self,
        location_id: Uuid,
    ) -> impl Future<Output = Result<bool, DomainError>> + Send;
}

pub trait ZoneRepository: Send + Sync {
    /// Tạo khu vực mới.
    fn create(
        &self,
        new_zone: NewZone,
    ) -> impl Future<Output = Result<Zone, DomainError>> + Send;

    /// Tìm khu vực theo id (chỉ zone chưa xoá mềm).
    fn find_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<Zone>, DomainError>> + Send;

    /// Danh sách khu vực (lọc + phân trang + sắp xếp); trả về (items, tổng số khớp).
    fn list(
        &self,
        filter: ZoneFilter,
    ) -> impl Future<Output = Result<(Vec<Zone>, i64), DomainError>> + Send;

    /// Cập nhật một phần; None nếu zone không tồn tại / đã xoá mềm.
    fn update(
        &self,
        id: Uuid,
        changes: ZoneUpdate,
    ) -> impl Future<Output = Result<Option<Zone>, DomainError>> + Send;

    /// Xoá mềm (set deleted_at). NotFound nếu zone không tồn tại / đã xoá.
    fn soft_delete(&self, id: Uuid) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Còn kệ (bin) nào CHƯA xoá mềm thuộc khu vực này không (để chặn xoá zone).
    fn has_active_bins(
        &self,
        zone_id: Uuid,
    ) -> impl Future<Output = Result<bool, DomainError>> + Send;
}

pub trait BinRepository: Send + Sync {
    /// Tạo kệ mới.
    fn create(&self, new_bin: NewBin) -> impl Future<Output = Result<Bin, DomainError>> + Send;

    /// Tìm kệ theo id (chỉ bin chưa xoá mềm).
    fn find_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<Bin>, DomainError>> + Send;

    /// Danh sách kệ (lọc + phân trang + sắp xếp); trả về (items, tổng số khớp).
    fn list(
        &self,
        filter: BinFilter,
    ) -> impl Future<Output = Result<(Vec<Bin>, i64), DomainError>> + Send;

    /// Cập nhật một phần; None nếu bin không tồn tại / đã xoá mềm.
    fn update(
        &self,
        id: Uuid,
        changes: BinUpdate,
    ) -> impl Future<Output = Result<Option<Bin>, DomainError>> + Send;

    /// Xoá mềm (set deleted_at). NotFound nếu bin không tồn tại / đã xoá.
    fn soft_delete(&self, id: Uuid) -> impl Future<Output = Result<(), DomainError>> + Send;
}

pub trait PasswordTokenRepository: Send + Sync {
    fn create(
        &self,
        token: NewPasswordToken,
    ) -> impl Future<Output = Result<PasswordToken, DomainError>> + Send;

    /// Token còn hiệu lực theo hash: chưa dùng (used_at IS NULL) và chưa hết hạn.
    fn find_active_by_hash(
        &self,
        token_hash: &str,
    ) -> impl Future<Output = Result<Option<PasswordToken>, DomainError>> + Send;

    /// Vô hiệu mọi token còn hiệu lực của user theo loại (đặt used_at = NOW()).
    fn invalidate_active(
        &self,
        user_id: Uuid,
        token_type: PasswordTokenType,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;
}

pub trait UserSessionRepository: Send + Sync {
    fn create(
        &self,
        new_session: NewSession,
    ) -> impl Future<Output = Result<Session, DomainError>> + Send;

    fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> impl Future<Output = Result<Option<Session>, DomainError>> + Send;

    /// Thu hồi một phiên cụ thể (logout). No-op nếu phiên đã thu hồi trước đó.
    fn revoke(
        &self,
        id: Uuid,
        reason: &str,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Thu hồi tất cả phiên còn hiệu lực của một user (logout mọi thiết bị).
    fn revoke_all_for_user(
        &self,
        user_id: Uuid,
        reason: &str,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Cập nhật `expires_at` cho phiên (sliding). No-op nếu phiên đã thu hồi.
    fn touch_expires(
        &self,
        id: Uuid,
        expires_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;
}
