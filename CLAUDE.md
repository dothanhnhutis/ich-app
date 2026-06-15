# ich-app — Context dự án (handoff)

> File này Claude Code **tự nạp mỗi phiên**. Mục tiêu: tiếp tục dự án trên bất kỳ máy nào.
> Cập nhật lần cuối: 2026-06-16 (admin: **đăng nhập thật + protected routes + CRUD vai trò + TanStack Query**; trước đó: doc/data-postgresql.md, CRUD item master + BOM/bom_lines, vendor, warehouse).

## 1. Tổng quan
App Rust theo kiến trúc **clean/hexagonal** — RBAC auth + quản lý kho (warehouse) + nhà cung cấp (vendor) + **item master/BOM** (sản xuất mỹ phẩm). Giao tiếp tiếng Việt; message lỗi trả về tiếng Việt.

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
- **Convention soft-delete**: cột `deleted_at TIMESTAMPTZ(3)` + **partial unique index** `… WHERE deleted_at IS NULL` (cho phép tái dùng code/name sau xoá mềm). Áp cho `users, roles, files, locations, warehouse_zones, storage_bins, vendors, items, item_uom_conversions, boms, bom_lines`.
- **Volume đã init rồi** thì migration KHÔNG tự chạy lại → áp schema bằng tay: `docker exec postgres_container psql -U admin -d pgdb -c "…"`. (Trên máy mới, volume trống → chạy đủ migrations + seed, không cần thao tác tay.)
- ⚠️ **Item master/BOM mới thêm vào `001_init` nhưng volume local có thể đã init từ trước** → trên volume cũ phải **áp tay (Live-apply)**: `UPDATE items SET type='RAW_MATERIAL' WHERE type='CHEMICAL'` → `ALTER TABLE items` (10 cột mới + CHECK + đổi `uq_items_sku` sang partial index) → `CREATE TABLE boms` + `bom_lines` (kèm index + FK RESTRICT) → **chạy lại DO-block của `002_trigger.sql`** để cấp audit/updated_at cho 2 bảng mới (002 quét 1 lần, không tự phủ bảng thêm sau) → `UPDATE permissions SET code='RAW_MATERIAL_*'`. Volume MỚI: tự chạy đủ, không thao tác tay.

## 5. Auth & session
- **Cache-first Redis**: `authenticate()` đọc `(session, user)` từ cache tới ~1h (`SESSION_CACHE_TTL_SECS`); chỉ chặn `Deactivated` theo **status đã cache**. ⇒ Deactivate/xoá user trong DB **không hiệu lực ngay** trừ khi gọi `auth_service.logout_all(user_id)` (revoke phiên + clear cache). Handler PATCH→DEACTIVATED và DELETE user đã gọi `logout_all`.
- **Token**: `SessionToken::generate()` → `{raw, hash=sha256(raw)}`; argon2 cho password.
- ⚠️ **KHÔNG xoá các `println!` debug trong `libs/application/src/services/auth_service.rs::login()`** (cố ý của chủ dự án).

## 6. RBAC
- Middleware `apps/api/src/middlewares/authz.rs`: `require_<RESOURCE>_<ACTION>` → `ensure(state, user_id, "CODE")` nạp permission **mỗi request** (không cache authz).
- Permission code dạng `RESOURCE_ACTION` (vd `LOCATION_VIEW`, `ZONE_CREATE`, `VENDOR_UPDATE`, `BOM_CREATE`). Danh mục đầy đủ ở `data/permissions.csv`. ⚠️ `CHEMICAL_*` đã đổi → `RAW_MATERIAL_*` (giữ UUID).
- ⚠️ **Item authz = PER-TYPE** (5 bộ: `RAW_MATERIAL_*`, `PACKAGING_*`, `UTILITY_*`, `SEMI_FINISHED_*`, `FINISHED_GOODS_*` — 20 quyền). Vì `type` nằm trong body (create) / trong row (update/delete) → **KHÔNG** dùng được route-middleware tĩnh → kiểm ở **TẦNG SERVICE**: handler `/items` nạp `user_service.permission_codes(user_id)` rồi truyền `&[String]` vào `ItemService` (mỗi method tự đòi `{TYPE}_{ACTION}`). **BOM** dùng 1 bộ `BOM_*` qua middleware chuẩn `require_bom_*`. Tất cả 24 quyền (20 + 4 BOM) đã gán super-admin.
- Super-admin role id `019c0cd2-374b-795a-a028-460024d912b7`. User test có đủ quyền: `gaconght@gmail.com` (**mật khẩu chưa biết** — cần để test luồng đăng nhập).

## 7. Module nghiệp vụ
Trạng thái: **Warehouse (location/zone/bin)** ✅ CRUD · **Vendor** ✅ CRUD · **Item Master** ✅ CRUD (authz per-type) · **BOM + bom_lines** ✅ CRUD (Hybrid).

### 7.1 Warehouse — CRUD location/zone/bin ✅
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

### 7.2 Vendor — CRUD nhà cung cấp ✅
Resource phẳng (không phân cấp), theo đúng "công thức" mục 2. Enum `VendorType` (SUPPLIER | MANUFACTURER | BOTH; `as_str()`+`FromStr`, parse ở service → 400 nếu sai). Field: `code, name, vendor_type, tax_code, address, phone, email, notes`. Soft-delete + partial unique (`code`); lỗi map theo `db.constraint()`.

| Method | Path | Quyền |
|---|---|---|
| POST / GET | `/api/v1/vendors` | VENDOR_CREATE / VENDOR_VIEW |
| GET / PATCH / DELETE | `/api/v1/vendors/{id}` | VENDOR_VIEW / VENDOR_UPDATE / VENDOR_DELETE |

GET list: `/vendors?code=&name=&vendor_type=&sort=code:asc,created_at:desc&page=1&page_size=20`.

### 7.3 Item Master ✅ + BOM/bom_lines ✅ (Hybrid)
`items` là master **dùng chung** (nguyên liệu thô, bao bì cấp 1/2/3 + thùng, dụng cụ/quà tặng, bán thành phẩm, thành phẩm); 1 item có **0, 1 hoặc nhiều** BOM. CRUD đầy đủ theo công thức mục 2.

| Method | Path | Quyền |
|---|---|---|
| POST / GET | `/api/v1/items` | `{TYPE}_CREATE` / `{TYPE}_VIEW` (per-type, ở service) |
| GET / PATCH / DELETE | `/api/v1/items/{id}` | `{TYPE}_VIEW` / `{TYPE}_UPDATE` / `{TYPE}_DELETE` |
| POST / GET | `/api/v1/boms` | BOM_CREATE / BOM_VIEW |
| GET / PATCH / DELETE | `/api/v1/boms/{id}` | BOM_VIEW / BOM_UPDATE / BOM_DELETE |
| POST | `/api/v1/boms/{id}/lines` | BOM_UPDATE |
| PATCH / DELETE | `/api/v1/boms/{id}/lines/{line_id}` | BOM_UPDATE |

- **items** (`ItemService<IR>`, **không** route_layer — authz per-type ở service): field Rust `item_type` ↔ cột `type` (`#[sqlx(rename="type")]` + `#[serde(rename="type")]`); `packaging_level` (chỉ PACKAGING); 5 flags (`is_lot_controlled` default TRUE qua `#[serde(default=...)]`); `density_g_ml/shelf_life_days/pao_months/inci_name/cas_number`. **`type` & `base_uom` BỊ KHOÁ** (immutable — không có ở `UpdateItemRequest`). GET list chỉ trả các loại có `{TYPE}_VIEW`. DELETE chặn nếu `is_referenced` (boms.output / bom_lines.component / item_uom_conversions / vendor_items).
- **BOM Hybrid** (`BomService<BR,IR>`): `POST /boms` mang `lines:[...]` → header+lines trong **1 transaction** (`create_with_lines`); `GET /boms/{id}` trả `{bom, lines}` nested; thêm/sửa/xoá từng line qua sub-route. `DELETE /boms` **cascade** xoá mềm lines (transaction). `output_item_id`/`bom_type` khoá; `component_item_id` của line khoá.
- **Enforcement app-level (đã làm, KHÔNG trigger):** output item phải `type ∈ {SEMI_FINISHED,FINISHED_GOODS}`; component phải tồn tại; cấm self-ref; **cycle đệ quy** qua recursive CTE `would_create_cycle(component, output)` (verify transitive trên DB thật); `uq_boms_default_active` ép 1 BOM default-ACTIVE/item/loại. Lock `base_uom` (khi có `inventory_transactions`) = follow-up.
- ⚠️ Cột số (`density_g_ml`, `output_qty`, `quantity`, `scrap_pct`, `input_qty`) đã đổi **DECIMAL → DOUBLE PRECISION** trong `001_init.sql` (khớp `f64` toàn app + cột zone; sqlx **chưa bật** decimal). Đổi sang exact-decimal sau = phải bật feature + đổi DTO sang `Decimal`.

## 8. Đã verify / CHƯA verify
- ✅ `cargo check --workspace` + `cargo clippy --all-targets` sạch (gồm items + boms/bom_lines).
- ✅ **SQL item/BOM verify trên DB throwaway thật** (PG18, áp `001_init`): cycle CTE đúng kể cả **transitive** (M→O→C); `is_referenced` đúng (component/output/uom/vendor); `type = ANY()` lọc per-type; đọc `f64` + cột `type` OK; CHECK `chk_items_pkg_level` + unique `uq_bom_lines_component` chặn đúng (tên constraint **khớp** `map_sqlx_err`). Schema (gồm DECIMAL→DOUBLE) áp sạch.
- ✅ (Đợt trước) schema fresh-volume + pg_partman: 001→003 sạch, `002` cấp trigger audit/updated_at cho boms/bom_lines.
- ❌ **CHƯA test luồng CÓ ĐĂNG NHẬP qua HTTP** (login → POST `/items` per-type → POST `/boms` kèm lines → thử cycle/guard/trùng/403/404). Cần mật khẩu `gaconght@gmail.com` hoặc session test tạm.
- ❌ **CHƯA áp lên DB local đang dùng** (volume cũ — Live-apply mục 4; nay gồm **20 permission mới** + **5 cột DOUBLE**).

## 9. Gotchas / quy ước
- Entity **bỏ field `deleted_at`** (chỉ lọc SQL; `*_COLS`/RETURNING liệt kê đúng cột).
- PATCH cột **nullable** (`address`, `temp_min_c`, `temp_max_c`, `humidity_max_pct`) **chưa set về NULL được** (COALESCE giữ giá trị cũ khi gửi rỗng/None — v1; cần sentinel để clear).
- axum 0.8: route tĩnh (`/users/me`) và động (`/users/{id}`) cùng tồn tại OK; `/x/{id}` ở nhiều nhóm method (view/update/delete) merge OK.
- Lỗi "còn con" trả `AppError::Validation` (HTTP **400**) — codebase **chưa có 409 Conflict**.
- `cargo run` cần `SERVER_PORT` (không có default → panic nếu thiếu).
- **Item per-type authz ở TẦNG SERVICE** (không route-middleware): handler nạp `permission_codes` → truyền `&[String]` vào `ItemService`; BOM thì middleware chuẩn. Resource mới mà quyền phụ thuộc DỮ LIỆU (vd theo `type`) → theo mẫu item này.
- Cột số item/bom = **DOUBLE PRECISION** + `f64` (sqlx chưa bật decimal; giống cột zone). ĐỪNG dùng DECIMAL với bind `f64`.
- Immutable (không có ở Update DTO): item `type`/`base_uom`; bom `output_item_id`/`bom_type`; bom_line `component_item_id`.
- ⚠️ **SQL `col IN (...)` = NULL khi `col IS NULL`** (không phải FALSE) → một CHECK kiểu `(type='PACKAGING' AND col IN (...))` sẽ **PASS** khi `col` NULL. Phải thêm `col IS NOT NULL AND …` (bài học từ `chk_items_pkg_level` — từng hở, đã sửa). Quy tắc: CHECK chỉ FAIL khi biểu thức = FALSE; NULL coi như pass.

## 10. Việc còn lại (next steps)
1. **Test e2e CÓ ĐĂNG NHẬP (HTTP)** cho item/BOM + warehouse + vendor (mục 8) — login, per-type 403, cycle/guard, trùng mã. **Ưu tiên cao.**
2. **Áp Live-apply** lên DB local (mục 4) hoặc rebuild volume; thêm `base_uom` vào `data/items.csv` + bật `COPY items` ở `005_seed.sql` **trước khi** seed item.
3. Mở rộng: `item_uom_conversions` CRUD; `vendor_items` (mua hàng: vendor_sku/is_preferred/purchase_uom/lead_time/MOQ) + `vendor_item_prices`; `item_lots` (density override + qc_status QUARANTINE/RELEASED/REJECTED + FEFO) → inventory; lock `base_uom` thực khi có `inventory_transactions`; cho phép đổi item `type` (kèm re-validate authz/packaging).
4. Cân nhắc: `AppError::Conflict` (409); PATCH set NULL cột nullable (sentinel/`Option<Option<T>>`); tách `001_init.sql` khi quá lớn.
5. **Frontend** (mục 12): đã có đăng nhập + protected routes + CRUD vai trò (react-query). Cần **tài khoản có mật khẩu** để e2e luồng login → /roles (tạo/sửa/xoá + chọn quyền). Tiếp: nối CRUD các resource còn lại (warehouse/vendor/item/BOM) theo mẫu `/roles`; hiển thị quyền người dùng (`useHasPermission`) thay chỗ hardcode `canUpdate/canDelete`.

## 11. Bản đồ file chính
- **Domain**: `libs/domain/src/entities/{location,zone,bin,vendor,item,bom,user,role}.rs`, `repositories.rs` (traits), `errors.rs` (`DomainError`).
- **Infra**: `libs/infrastructure/src/repositories/pg_{location,zone,bin,vendor,item,bom,user,role}_repository.rs`.
- **Application**: `libs/application/src/dto/{location,zone,bin,vendor,item,bom}_dto.rs`, `services/{location,zone,bin,vendor,item,bom}_service.rs`, `errors.rs` (`AppError`).
- **API**: `apps/api/src/{main.rs, errors.rs, extractor.rs}`, `middlewares/{auth.rs, authz.rs}`, `handlers/*_handler.rs`, `routes/*_route.rs`.
- **DB chưa có code .rs**: `item_uom_conversions` (trong `001_init.sql`) — CRUD để follow-up.
- **Shared/DB**: `libs/shared/src/config.rs`, `migrations/00{1..5}_*.sql`, `data/*.csv`, `docker-compose.dev.yaml`.
- **Frontend**: `apps/admin/src/{main.tsx, App.tsx(router context+providers), routes/{__root,index,login}.tsx, routes/_protected/{route,dashboard,profile,roles}.tsx, components/{auth-provider,app-sidebar,nav-*,role-form-sheet,delete-role-dialog,table-pagenation,StatusBadge,ThemeProvider}.tsx, components/ui/*, contexts/{auth-context,theme-context}.ts, lib/{api,utils}.ts, hooks/use-theme.ts, vite.config.ts(proxy)}` (xem mục 12).
- **Docs**: `doc/data-postgresql.md` (mô tả schema PostgreSQL — 19 bảng, ERD, quy ước, enum, FK, seed).

## 12. Frontend admin (apps/admin) — auth + role CRUD ✅
SPA quản trị, **tách hẳn** backend Rust. Stack: **React 19 + Vite 8 + TanStack Router** (file-based; `routeTree.gen.ts` do `@tanstack/router-plugin` tự sinh) + **TanStack React Query** (data layer) + **TanStack React Form** + **Zod** + **Tailwind v4** + **shadcn/ui** (`@base-ui/react`, style base-luma) + lucide-react + React Compiler. Quản lý gói: **pnpm**.

- **Kết nối API (dev)**: **Vite proxy** `/api` → `http://localhost:4555` (`vite.config.ts`) → same-origin nên cookie `session` **httpOnly** hoạt động (KHÔNG cần CORS). ⚠️ Dev cần backend `COOKIE_SECURE=false` (cookie qua http). Client **`lib/api.ts`**: `fetch` `credentials:"include"` + class `ApiError`; base = `import.meta.env.VITE_API_URL ?? ""` (set `VITE_API_URL` để switch direct/prod). Endpoint: `api.login/me/logout`, `api.roles.{list,create,update,remove,permissionsOf}`, `api.permissions.list`.
- **Auth**: `components/auth-provider.tsx` (`AuthContext` ở `contexts/auth-context.ts`) giữ `{profile, hydrating}` + `login/logout`; mount → hydrate qua `GET /users/me`. `App.tsx` bơm `auth` vào **router context** + **hydration-gating** (hiện splash khi `hydrating` rồi mới render `RouterProvider` → `beforeLoad` luôn thấy trạng thái đã settle, F5 không bị đá ra). `routes/_protected/route.tsx` `beforeLoad` chặn chưa đăng nhập → `/login?redirect=`; `/login` (`login.tsx`) gọi `api.login` → `/profile`, và tự về `/profile` nếu đã đăng nhập; đăng xuất ở `nav-user`.
- **Data layer**: `QueryClientProvider` ở `App.tsx`. List dùng `useQuery` (vd key `["roles",{page,page_size}]`) + `keepPreviousData` (đổi trang không nháy skeleton, quay lại trang đã xem là tức thì); **invalidate** key sau tạo/sửa/xoá (thay refetch tay).
- **CRUD vai trò** (`/roles`, `routes/_protected/roles.tsx`) — mẫu cho các trang list sau: bảng (`components/ui/table`) + **tổng số** + dropdown **page-size** (10/20/50/100, đổi → về trang 1); component `table-pagenation` nhận **`onPageChange`** (route sở hữu search param, type-safe). Tạo/Sửa qua `RoleFormSheet` (Sheet + React-Form/Zod, **chọn quyền theo nhóm** từ `GET /permissions`; Sửa pre-check `GET /roles/{id}/permissions`; bắt buộc ≥1 quyền); Xoá xác nhận (`DeleteRoleDialog`); tôn trọng `can_update`/`can_delete` (role hệ thống ẩn nút). `StatusBadge` hiển thị trạng thái.
- **Cấu trúc**: `routes/{__root(router context),index,login}.tsx` + `routes/_protected/{route(sidebar+breadcrumb layout),dashboard,profile,roles}.tsx`; `components/{auth-provider,app-sidebar,nav-main,nav-user,role-form-sheet,delete-role-dialog,table-pagenation,StatusBadge}.tsx`; `components/ui/*` (button/field/input/label/separator/table/badge/pagination/sheet/dropdown-menu/sidebar/avatar/breadcrumb/collapsible/skeleton/tooltip); `lib/{api,utils(cn+calcPages)}.ts`; `contexts/{auth-context,theme-context}.ts`; `ThemeProvider` + `hooks/use-theme.ts`.
- **Chạy**: `cd apps/admin && pnpm install && pnpm dev` (:5173). Build: `pnpm build` (`tsc -b && vite build`) — **xanh**. `.tanstack/` đã gitignore.
- **Trạng thái**: auth + role CRUD **chạy được (build xanh)**; ❌ **CHƯA e2e với đăng nhập thật** — cần tài khoản có mật khẩu (mật khẩu `gaconght@gmail.com` chưa biết).
