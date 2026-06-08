# ich-app — Context dự án (handoff)

> File này Claude Code **tự nạp mỗi phiên**. Mục tiêu: tiếp tục dự án trên bất kỳ máy nào.
> Cập nhật lần cuối: 2026-06 (sau khi xong CRUD warehouse location/zone/bin).

## 1. Tổng quan
App Rust theo kiến trúc **clean/hexagonal** — RBAC auth + quản lý kho (warehouse). Giao tiếp tiếng Việt; message lỗi trả về tiếng Việt.

- **Workspace Cargo**: `libs/{domain, application, infrastructure, shared}` + `apps/{api, worker, iops, admin}`.
- **Stack**: Rust **edition 2024** (let-chains `if let … && …`), **axum 0.8** (route param cú pháp `{id}` KHÔNG phải `:id`), **sqlx 0.8** (runtime `query_as`/`query_scalar`, `FromRow`, transaction `pool.begin()`), `validator`, `uuid` v7, `chrono`, `serde`. Redis (cache phiên), RabbitMQ/lapin (email job), Gmail OAuth2 (worker gửi mail).
- **Phân lớp phụ thuộc**: `domain` (entity + trait repo, không phụ thuộc gì) ← `application` (service + dto + AppError) ← `infrastructure` (impl repo Postgres/Redis/Rabbit) ← `apps/api` (axum, wiring). `shared` = config/messaging dùng chung.

## 2. Công thức thêm 1 resource (RẤT QUAN TRỌNG — theo đúng mẫu này)
Mẫu chuẩn: stack **`location`** (đơn giản) và **`zone`** (có enum + cột số + FK). Khi thêm resource `X`:

1. **Domain** `libs/domain/src/entities/x.rs`: `X`, `NewX`, `XUpdate` (`#[derive(Default)]`, field `Option`), `XFilter`, `XSort`/`XSortField` (FromStr), enum nếu cần (mẫu `ZoneType`/`RoleStatus`: `as_str()` + `FromStr`). **Entity KHÔNG có field `deleted_at`** (chỉ lọc trong SQL). Khai báo ở `entities/mod.rs` (`mod x; pub use x::*;`). Tái dùng `SortDir` từ `entities`.
2. **Trait** `libs/domain/src/repositories.rs`: `trait XRepository` (RPITIT: `fn f(&self,..) -> impl Future<Output=Result<T, DomainError>> + Send`). Thêm import entity.
3. **Infra** `libs/infrastructure/src/repositories/pg_x_repository.rs`: `XRow` (`sqlx::FromRow`) + `From`/`TryFrom` (TryFrom nếu có enum), `map_sqlx_err` (map theo `db.constraint()` cho message thân thiện), `order_by_clause` (whitelist match → literal, luôn nối `id DESC`), lọc optional `($n::type IS NULL OR col=$n)`, update bằng `COALESCE($n, col)`, soft_delete (`UPDATE … SET deleted_at=NOW() WHERE id=$1 AND deleted_at IS NULL`, `rows_affected()==0` → NotFound). Khai báo ở `repositories/mod.rs`.
4. **Application** `dto/x_dto.rs` (`CreateX`/`UpdateX`/`ListXQuery`/`XResponse`, `#[serde(deny_unknown_fields)]`, `validator`) + `services/x_service.rs` (`XService<XR, …>`; helper `norm`/`parse_sort`; validate → repo). Khai báo ở `dto/mod.rs`, `services/mod.rs`.
5. **API**: `apps/api/src/middlewares/authz.rs` thêm `require_x_<action>` (gọi `ensure(state, user_id, "X_ACTION")`); `handlers/x_handler.rs`; `routes/x_route.rs` (4 nhóm `route_layer` create/view/update/remove rồi `.merge`). Khai báo ở `handlers/mod.rs`, `routes/mod.rs` (+ `.merge(x_route::routes::<S>(state.clone()))`).
6. **Wiring** `apps/api/src/main.rs`: `let x_repo = PgXRepository::new(pool.clone())`; `let x_service = Arc::new(XService::new(x_repo, …))`; field `x_service` trong `AppState` + dòng init. `PgXRepository` là `Clone` (pool `Arc` bên trong) → clone repo khi service con cần repo cha.

## 3. Chạy local
```bash
# 1. Hạ tầng (postgres_container, redis_container, rabbitmq_container)
docker compose -f docker-compose.dev.yaml up -d
# 2. Tạo .env (KHÔNG có trong git — xem mục biến env bên dưới)
# 3. API (SERVER_PORT BẮT BUỘC — config không có default)
cargo run -p api          # hoặc: SERVER_PORT=4555 cargo run -p api
cargo run -p worker       # consumer email (cần GMAIL_*)
# Kiểm tra nhanh: cargo check --workspace && cargo clippy --workspace --all-targets
```
**Biến `.env` cần có** (giá trị là bí mật, không commit):
`DATABASE_URL, REDIS_URL, RABBITMQ_URL, RABBITMQ_EMAIL_QUEUE, APP_WEB_URL, SERVER_HOST, SERVER_PORT, COOKIE_SECURE, COOKIE_DOMAIN, CORS_ALLOWED_ORIGINS, SESSION_TTL_SECS, SESSION_CACHE_TTL_SECS, SESSION_DB_SYNC_SECS, PASSWORD_TOKEN_TTL_SECS, RESET_PASSWORD_TOKEN_TTL_SECS` + (worker) `GMAIL_CLIENT_ID, GMAIL_CLIENT_SECRET, GMAIL_REFRESH_TOKEN, GMAIL_SENDER`.

Smoke test routing (không cần login): GET `/api/v1/<resource>` chưa auth → **401**; path lạ → **404**.

## 4. DB & migrations
- `migrations/00{1..5}_*.sql` chạy qua `docker-entrypoint-initdb.d` **chỉ khi volume MỚI** (theo thứ tự alphabet): `001_init` (schema) → `002_trigger` (set_updated_at + audit — **DO block quét động, tự phủ MỌI bảng**, không cần đăng ký tay) → `003_partition` (pg_partman cho audit_logs) → `004_test` → `005_seed`.
- **Seed**: `005_seed.sql` `COPY` từ `data/*.csv` (mount `./data:/tmp`). `data/permissions.csv` + `data/role_permissions.csv` là **NGUỒN SỰ THẬT** (id UUID tường minh). ⚠️ Đừng `INSERT` trùng permission ở `001_init` — sẽ vỡ COPY ở `005` khi rebuild volume.
- **Convention soft-delete**: cột `deleted_at TIMESTAMPTZ(3)` + **partial unique index** `… WHERE deleted_at IS NULL` (cho phép tái dùng code/name sau xoá mềm). Áp cho `users, roles, files, locations, warehouse_zones, storage_bins`.
- **Volume đã init rồi** thì migration KHÔNG tự chạy lại → áp schema bằng tay: `docker exec postgres_container psql -U admin -d pgdb -c "…"`. (Trên máy mới, volume trống → chạy đủ migrations + seed, không cần thao tác tay.)

## 5. Auth & session
- **Cache-first Redis**: `authenticate()` đọc `(session, user)` từ cache tới ~1h (`SESSION_CACHE_TTL_SECS`); chỉ chặn `Deactivated` theo **status đã cache**. ⇒ Deactivate/xoá user trong DB **không hiệu lực ngay** trừ khi gọi `auth_service.logout_all(user_id)` (revoke phiên + clear cache). Handler PATCH→DEACTIVATED và DELETE user đã gọi `logout_all`.
- **Token**: `SessionToken::generate()` → `{raw, hash=sha256(raw)}`; argon2 cho password.
- ⚠️ **KHÔNG xoá các `println!` debug trong `libs/application/src/services/auth_service.rs::login()`** (cố ý của chủ dự án).

## 6. RBAC
- Middleware `apps/api/src/middlewares/authz.rs`: `require_<RESOURCE>_<ACTION>` → `ensure(state, user_id, "CODE")` nạp permission **mỗi request** (không cache authz).
- Permission code dạng `RESOURCE_ACTION` (vd `LOCATION_VIEW`, `ZONE_CREATE`, `USER_DELETE`). Danh mục đầy đủ ở `data/permissions.csv`.
- Super-admin role id `019c0cd2-374b-795a-a028-460024d912b7`. User test có đủ quyền: `gaconght@gmail.com` (**mật khẩu chưa biết** — cần để test luồng đăng nhập).

## 7. Module Warehouse — TRẠNG THÁI: CRUD location/zone/bin XONG ✅
Phân cấp: **location** (kho) → **zone** (`warehouse_zones`, khu vực) → **bin** (`storage_bins`, kệ). FK `ON DELETE RESTRICT`.

| Method | Path | Quyền |
|---|---|---|
| POST / GET | `/api/v1/locations` | LOCATION_CREATE / LOCATION_VIEW |
| GET / PATCH / DELETE | `/api/v1/locations/{id}` | LOCATION_VIEW / LOCATION_UPDATE / LOCATION_DELETE |
| POST / GET | `/api/v1/zones` | ZONE_CREATE / ZONE_VIEW |
| GET / PATCH / DELETE | `/api/v1/zones/{id}` | ZONE_VIEW / ZONE_UPDATE / ZONE_DELETE |
| POST / GET | `/api/v1/bins` | BIN_CREATE / BIN_VIEW |
| GET / PATCH / DELETE | `/api/v1/bins/{id}` | BIN_VIEW / BIN_UPDATE / BIN_DELETE |

GET list hỗ trợ lọc + phân trang + sắp xếp: `/zones?location_id=&name=&zone_type=&sort=name:asc,created_at:desc&page=1&page_size=20`; `/bins?zone_id=&code=&name=&sort=&page=&page_size=`; `/locations?code=&name=&sort=&page=&page_size=`.

**Đặc tả nghiệp vụ đã chốt:**
- **Chặn xoá khi còn con (cả 2 tầng)**: DELETE location còn zone / DELETE zone còn bin (con **chưa xoá mềm**) → **400**. Dùng `has_active_zones`/`has_active_bins` (`SELECT EXISTS … WHERE deleted_at IS NULL`) vì soft-delete là UPDATE nên FK RESTRICT không kích hoạt. Con đã xoá mềm KHÔNG chặn.
- **Cho đổi cha**: PATCH zone đổi `location_id`, PATCH bin đổi `zone_id` — kiểm cha mới còn sống qua `parent_repo.find_by_id` (bắt được cả cha đã xoá mềm; `ZoneService<ZR,LR>`, `BinService<BR,ZR>`).
- **Lỗi thân thiện** map theo `db.constraint()`: unique (tên/mã trùng), CHECK `chk_warehouse_zones_temp_range` (min≤max) / `chk_warehouse_zones_humidity` (0–100), FK (cha không tồn tại). `zone_type` (7 giá trị: FINISHED_GOODS, RAW_MATERIAL, PACKAGING, QUARANTINE, REJECT, RETURN, UTILITY) parse ở service → 400 nếu sai.

## 8. Đã verify / CHƯA verify
- ✅ `cargo check --workspace` + `cargo clippy --all-targets` sạch; server boot không panic (route merge OK); routing 401 (chưa auth) / 404 (path lạ).
- ❌ **CHƯA test luồng CÓ ĐĂNG NHẬP** (tạo location→zone→bin, thử xoá cha bị chặn, trùng mã/tên, đổi cha, 403 khi thiếu quyền). Cần mật khẩu `gaconght@gmail.com` hoặc tạo session test tạm rồi xoá.

## 9. Gotchas / quy ước
- Entity **bỏ field `deleted_at`** (chỉ lọc SQL; `*_COLS`/RETURNING liệt kê đúng cột).
- PATCH cột **nullable** (`address`, `temp_min_c`, `temp_max_c`, `humidity_max_pct`) **chưa set về NULL được** (COALESCE giữ giá trị cũ khi gửi rỗng/None — v1; cần sentinel để clear).
- axum 0.8: route tĩnh (`/users/me`) và động (`/users/{id}`) cùng tồn tại OK; `/x/{id}` ở nhiều nhóm method (view/update/delete) merge OK.
- Lỗi "còn con" trả `AppError::Validation` (HTTP **400**) — codebase **chưa có 409 Conflict**.
- `cargo run` cần `SERVER_PORT` (không có default → panic nếu thiếu).

## 10. Việc còn lại (next steps)
1. **Test e2e có auth** cho warehouse (ưu tiên — phần chưa verify ở mục 8).
2. Cân nhắc thêm `AppError::Conflict` (409) cho lỗi "còn con".
3. PATCH cho phép set NULL cột nullable (sentinel/`Option<Option<T>>`).
4. Resource kế tiếp: theo đúng "công thức" mục 2.

## 11. Bản đồ file chính
- **Domain**: `libs/domain/src/entities/{location,zone,bin,user,role}.rs`, `repositories.rs` (traits), `errors.rs` (`DomainError`).
- **Infra**: `libs/infrastructure/src/repositories/pg_{location,zone,bin,user,role}_repository.rs`.
- **Application**: `libs/application/src/dto/{location,zone,bin}_dto.rs`, `services/{location,zone,bin}_service.rs`, `errors.rs` (`AppError`).
- **API**: `apps/api/src/{main.rs, errors.rs, extractor.rs}`, `middlewares/{auth.rs, authz.rs}`, `handlers/*_handler.rs`, `routes/*_route.rs`.
- **Shared/DB**: `libs/shared/src/config.rs`, `migrations/00{1..5}_*.sql`, `data/*.csv`, `docker-compose.dev.yaml`.
