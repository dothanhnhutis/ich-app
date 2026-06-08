-- ==========================================================
-- Extension
-- ==========================================================
CREATE EXTENSION IF NOT EXISTS pgcrypto;
-- ==========================================================
-- Config
-- ==========================================================
ALTER DATABASE pgdb
    SET
        datestyle = 'ISO, DMY';

ALTER DATABASE pgdb
    SET
        timezone = 'Asia/Ho_Chi_Minh';


-- create user_sessions table
CREATE TABLE user_sessions
(
    id            UUID           NOT NULL DEFAULT uuidv7(),
    user_id       UUID           NOT NULL,
    token_hash    CHAR(64)       NOT NULL UNIQUE,
    device_id     VARCHAR(255),            -- Fingerprint do client tự tạo, dùng để nhận ra "cùng máy" dù đổi IP
    device_name   VARCHAR(255),            -- Human-readable: "Chrome 124 · Windows 11", "MyApp 2.1 · macOS 14"
    device_type   VARCHAR(20)    NOT NULL, -- 'web' | 'desktop' | 'mobile'
    platform      VARCHAR(100),            -- "Windows 11" | "macOS 14.5" | "Ubuntu 22.04"
    app_version   VARCHAR(50),             -- Chỉ có trên desktop app, null với web
    user_agent    TEXT,                    -- Raw User-Agent header, dùng để debug
    ip_address    INET,                    -- IP lúc login, dùng để hiển thị "đăng nhập từ đâu"

    revoked_at    TIMESTAMPTZ(3),
    revoke_reason VARCHAR(20),
    -- 'LOGOUT'  : user tự logout
    -- 'FORCED'  : user logout tất cả thiết bị
    -- 'USER'   : user thu hồi
    -- 'EXPIRED' : cleanup job đánh dấu sau khi hết hạn

    expires_at    TIMESTAMPTZ(3) NOT NULL,
    created_at    TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_revoke_reason
        CHECK (revoke_reason IN ('LOGOUT', 'FORCED', 'USER', 'EXPIRED')),
    CONSTRAINT pk_user_sessions PRIMARY KEY (id)
);


-- ==========================================================
-- Danh mục file
-- ==========================================================
CREATE TABLE IF NOT EXISTS files
(
    id            UUID           NOT NULL DEFAULT uuidv7(),
    original_name TEXT           NOT NULL, -- tên file người dùng upload
    mime_type     VARCHAR(100)   NOT NULL, -- loại file
    destination   TEXT           NOT NULL, -- đường dẫn ngắn đến file
    file_name     TEXT           NOT NULL, -- tên file
    path          TEXT           NOT NULL, -- đường dẫn đầy đủ đến file
    size          BIGINT         NOT NULL, -- kích thước file
    uploaded_by   UUID           NOT NULL, -- upload bởi ai
    deleted_at    TIMESTAMPTZ(3),          -- xoá lúc nào
    created_at    TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_files PRIMARY KEY (id)
);

-- ==========================================================
-- audit_logs table
-- ==========================================================
CREATE TABLE IF NOT EXISTS audit_logs
(
    id             UUID         NOT NULL DEFAULT uuidv7(), -- Dùng kiểu UUID thực thụ
    table_name     VARCHAR(100) NOT NULL,
    record_id      TEXT         NOT NULL,
    action         VARCHAR(10)  NOT NULL,                  -- INSERT, UPDATE, DELETE
    old_data       JSONB,
    new_data       JSONB,
    changed_by     TEXT         NOT NULL,
    transaction_id TEXT,
    changed_at     TIMESTAMPTZ(3)        DEFAULT NOW() NOT NULL,
    CONSTRAINT pk_audit_logs PRIMARY KEY (id, changed_at)  -- Phải bao gồm cột phân mảnh
) PARTITION BY RANGE (changed_at);

-- ==========================================================
-- Danh mục quyền và vai trò
-- ==========================================================
CREATE TABLE IF NOT EXISTS permissions
(
    id          UUID           NOT NULL DEFAULT uuidv7(),
    code        VARCHAR(100)   NOT NULL, -- vd: CHEMICAL_CREATE, PO_VIEW
    description TEXT           NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_permissions PRIMARY KEY (id),
    CONSTRAINT permissions_code_unique UNIQUE (code)
);

CREATE TABLE IF NOT EXISTS role_permissions
(
    role_id       UUID           NOT NULL,
    permission_id UUID           NOT NULL,
    created_at    TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_role_permissions PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE IF NOT EXISTS roles
(
    id             UUID           NOT NULL DEFAULT uuidv7(),
    name           VARCHAR(255)   NOT NULL,
    description    TEXT           NOT NULL DEFAULT '',
    status         VARCHAR(20)    NOT NULL DEFAULT 'ACTIVE', -- ACTIVE | DEACTIVATED
    deactivated_at TIMESTAMPTZ(3),                           -- vô hiệu hoá lúc nào
    deleted_at     TIMESTAMPTZ(3),                           -- xoá mềm
    can_delete     BOOLEAN        NOT NULL DEFAULT TRUE,
    can_update     BOOLEAN        NOT NULL DEFAULT TRUE,
    created_at     TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_roles PRIMARY KEY (id)
);

-- ==========================================================
-- Danh mục người dùng
-- ==========================================================
CREATE TABLE IF NOT EXISTS user_roles
(
    user_id    UUID           NOT NULL,
    role_id    UUID           NOT NULL,
    created_at TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_user_roles PRIMARY KEY (user_id, role_id)
);

CREATE TABLE IF NOT EXISTS users
(
    id                  UUID           NOT NULL DEFAULT uuidv7(),
    email               VARCHAR(255)   NOT NULL,
    password_hash       TEXT,
    username            VARCHAR(100),
    status              VARCHAR(20)    NOT NULL DEFAULT 'PENDING_PASSWORD', -- ACTIVE | DEACTIVATED | PENDING_PASSWORD
    deactivated_at      TIMESTAMPTZ(3),                                     -- vô hiệu hoá lúc nào
    deleted_at          TIMESTAMPTZ(3),                                     -- xoá mềm
    password_changed_at TIMESTAMPTZ(3),                                     -- lần cuối đổi mật khẩu
    created_at          TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_users PRIMARY KEY (id),
    CONSTRAINT chk_users_pending_password
    CHECK (
        status = 'PENDING_PASSWORD'
        OR username IS NOT NULL
        OR password_hash IS NOT NULL
    )
);

CREATE TABLE IF NOT EXISTS user_avatars
(
    file_id    UUID           NOT NULL,
    user_id    UUID           NOT NULL,
    width      INTEGER        NOT NULL,
    height     INTEGER        NOT NULL,
    is_primary BOOLEAN        NOT NULL DEFAULT FALSE, -- Hình đại diện
    deleted_at TIMESTAMPTZ(3),                        -- xoá lúc nào
    created_at TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_user_avatars PRIMARY KEY (file_id, user_id)
);

CREATE TABLE IF NOT EXISTS password_tokens
(
    id         UUID           NOT NULL DEFAULT uuidv7(),
    user_id    UUID           NOT NULL,
    token_hash TEXT           NOT NULL,
    type       VARCHAR(20)    NOT NULL, -- INIT | RESET-PASSWORD
    expires_at TIMESTAMPTZ(3) NOT NULL,
    used_at    TIMESTAMPTZ(3),
    created_at TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_password_tokens PRIMARY KEY (id),
    CONSTRAINT uq_password_tokens_token_hash UNIQUE (token_hash),
    CONSTRAINT chk_password_tokens_type CHECK (type IN ('INIT', 'RESET-PASSWORD'))
);


-- ==========================================================
-- QUẢN LÝ KHO (WAREHOUSE STRUCTURE)
-- ==========================================================

-- Vị trí kho vật lý (toà nhà, chi nhánh)
CREATE TABLE IF NOT EXISTS locations
(
    id         UUID           NOT NULL DEFAULT uuidv7(),
    code       VARCHAR(50)    NOT NULL,
    name       VARCHAR(150)   NOT NULL,
    address    VARCHAR(255),
    deleted_at TIMESTAMPTZ(3),
    created_at TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_locations PRIMARY KEY (id)
);

-- Khu vực trong kho (zone)
CREATE TABLE IF NOT EXISTS warehouse_zones
(
    id                  UUID           NOT NULL DEFAULT uuidv7(),
    location_id         UUID           NOT NULL,
    name                VARCHAR(150)   NOT NULL,
    zone_type           VARCHAR(50)    NOT NULL,
    temp_min_c          DOUBLE PRECISION,
    temp_max_c          DOUBLE PRECISION,
    humidity_max_pct    DOUBLE PRECISION,
    is_light_protected  BOOLEAN        NOT NULL DEFAULT FALSE,
    is_ventilated       BOOLEAN        NOT NULL DEFAULT FALSE,
    is_explosion_proof  BOOLEAN        NOT NULL DEFAULT FALSE,
    deleted_at          TIMESTAMPTZ(3),
    created_at          TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_warehouse_zones PRIMARY KEY (id),
    CONSTRAINT chk_warehouse_zones_type CHECK (zone_type IN (
                                                             'FINISHED_GOODS',
                                                             'RAW_MATERIAL',
                                                             'PACKAGING',
                                                             'QUARANTINE',
                                                             'REJECT',
                                                             'RETURN',
                                                             'UTILITY'
        )),
    CONSTRAINT chk_warehouse_zones_temp_range CHECK (
        temp_min_c IS NULL OR temp_max_c IS NULL OR temp_min_c <= temp_max_c
        ),
    CONSTRAINT chk_warehouse_zones_humidity CHECK (
        humidity_max_pct IS NULL OR (humidity_max_pct >= 0 AND humidity_max_pct <= 100)
        )
);

-- Ô/kệ lưu trữ trong khu vực
CREATE TABLE IF NOT EXISTS storage_bins
(
    id         UUID           NOT NULL DEFAULT uuidv7(),
    zone_id    UUID           NOT NULL,
    code       VARCHAR(50)    NOT NULL,
    name       VARCHAR(255)   NOT NULL,
    deleted_at TIMESTAMPTZ(3),
    created_at TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_storage_bins PRIMARY KEY (id)
);


-- ==========================================================
-- NHÀ CUNG CẤP (VENDORS)
-- ==========================================================
CREATE TABLE IF NOT EXISTS vendors
(
    id          UUID           NOT NULL DEFAULT uuidv7(),
    code        VARCHAR(50)    NOT NULL,                    -- mã nội bộ vendor
    name        VARCHAR(255)   NOT NULL,
    vendor_type VARCHAR(20)    NOT NULL DEFAULT 'SUPPLIER', -- SUPPLIER | MANUFACTURER | BOTH
    tax_code    VARCHAR(50),
    address     VARCHAR(255),
    phone       VARCHAR(50),
    email       VARCHAR(255),
    notes       TEXT,
    deleted_at  TIMESTAMPTZ(3),
    created_at  TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_vendors PRIMARY KEY (id),
    CONSTRAINT uq_vendors_code UNIQUE (code),
    CONSTRAINT chk_vendors_type CHECK (vendor_type IN ('SUPPLIER', 'MANUFACTURER', 'BOTH'))
);

CREATE TABLE IF NOT EXISTS vendor_items
(
    id         UUID           NOT NULL DEFAULT uuidv7(),
    vendor_id  UUID           NOT NULL,
    item_id    UUID           NOT NULL,
    created_at TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_vendor_items PRIMARY KEY (id),
    CONSTRAINT uq_vendor_items UNIQUE (vendor_id, item_id)
);
CREATE INDEX idx_vendor_items_item ON vendor_items (item_id);



-- ==========================================================
-- DANH MỤC VẬT TƯ (ITEMS)
-- ==========================================================
CREATE TABLE IF NOT EXISTS items
(
    id          UUID           NOT NULL DEFAULT uuidv7(),
    sku         VARCHAR(50)    NOT NULL, -- mã nội bộ
    name        VARCHAR(255)   NOT NULL,
    type        VARCHAR(20)    NOT NULL, -- CHEMICAL | PACKAGING | UTILITY | FINISHED_GOOD
    base_uom    VARCHAR(20)    NOT NULL, -- 'kg' | 'L' | 'pcs' | 'g' | 'mL' - LOCK khi đã có inventory_transactions
    description TEXT,
    deleted_at  TIMESTAMPTZ(3),
    created_at  TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_items PRIMARY KEY (id),
    CONSTRAINT uq_items_sku UNIQUE (sku),
    CONSTRAINT chk_items_type CHECK (type IN ('CHEMICAL', 'PACKAGING', 'UTILITY', 'FINISHED_GOOD'))
);
CREATE INDEX idx_items_type ON items (type) WHERE deleted_at IS NULL;
CREATE INDEX idx_items_name ON items (name) WHERE deleted_at IS NULL;



-- ==========================================================
-- CHUYỂN ĐỔI ĐƠN VỊ (UOM CONVERSION)
-- ==========================================================
-- Lưu các đơn vị quy đổi của 1 item (alternative ↔ base_uom).
-- Base UOM lưu ở items.base_uom — bảng này chỉ chứa các đơn vị thay thế.
-- Ví dụ: items.base_uom='kg', conversion: 'Phuy 250kg' → factor=250 (1 Phuy = 250 kg).
CREATE TABLE IF NOT EXISTS item_uom_conversions
(
    id                UUID           NOT NULL DEFAULT uuidv7(),
    item_id           UUID           NOT NULL,
    alternative_uom   VARCHAR(100)   NOT NULL, -- 'Phuy 250kg' | 'Thùng' | 'Bao'
    conversion_factor DECIMAL(15, 6) NOT NULL CHECK (conversion_factor > 0),
    is_purchase_uom   BOOLEAN        NOT NULL DEFAULT TRUE,
    deleted_at        TIMESTAMPTZ(3),
    created_at        TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_item_uom_conversions PRIMARY KEY (id)
);
-- Tránh trùng tên đơn vị trong cùng 1 item
CREATE UNIQUE INDEX uix_item_uom_name
    ON item_uom_conversions (item_id, alternative_uom)
    WHERE deleted_at IS NULL;




-- ==========================================================
-- Index
-- ==========================================================

-- create user_sessions index
CREATE INDEX idx_refresh_user ON user_sessions (user_id);
CREATE UNIQUE INDEX uq_idx_token_hash ON user_sessions (token_hash);
CREATE INDEX idx_user_revoked ON user_sessions (user_id, revoked_at);

-- create file index
CREATE INDEX idx_files_deleted_at ON files (deleted_at)
    WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX uix_files_path ON files (path)
    WHERE deleted_at IS NULL;

--create audit_logs index
CREATE INDEX IF NOT EXISTS idx_audit_logs_table_record ON audit_logs (table_name, record_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_tx ON audit_logs (transaction_id);

-- create roles index
CREATE INDEX IF NOT EXISTS idx_roles_status ON roles (status) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_roles_name_status_active ON roles (name, status) WHERE deleted_at IS NULL;

-- create users index
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email_unique ON users (email) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_users_status ON users (status) WHERE deleted_at IS NULL;

-- create user_avatars index
CREATE INDEX IF NOT EXISTS idx_user_avatars_selected ON user_avatars (is_primary) WHERE is_primary IS TRUE;

-- create password_tokens index
CREATE INDEX idx_password_tokens_user_id ON password_tokens (user_id);
CREATE INDEX idx_password_tokens_expires_at ON password_tokens (expires_at);

-- create warehouse_zones index
CREATE INDEX IF NOT EXISTS idx_warehouse_zones_location_id ON warehouse_zones (location_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_warehouse_zones_type ON warehouse_zones (zone_type) WHERE deleted_at IS NULL;
-- create storage_bins index
CREATE INDEX IF NOT EXISTS idx_storage_bins_zone_id ON storage_bins (zone_id) WHERE deleted_at IS NULL;

-- partial unique index cho warehouse (cho phép tái dùng code/name sau xoá mềm — đồng nhất users/roles/files)
CREATE UNIQUE INDEX IF NOT EXISTS uq_locations_code ON locations (code) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_warehouse_zones_loc_name ON warehouse_zones (location_id, name) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_storage_bins_code ON storage_bins (code) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_storage_bins_zone_name ON storage_bins (zone_id, name) WHERE deleted_at IS NULL;


-- ==========================================================
-- Khoá ngoại
-- ==========================================================

--- AddForeignKey password_tokens table
ALTER TABLE password_tokens
    ADD CONSTRAINT fk_password_tokens_user_id FOREIGN KEY (user_id)
        REFERENCES users (id) ON DELETE CASCADE;

--- AddForeignKey user_sessions table
ALTER TABLE user_sessions
    ADD CONSTRAINT fk_user_sessions_user_id FOREIGN KEY (user_id)
        REFERENCES users (id) ON DELETE CASCADE;

-- AddForeignKey role_permissions table
ALTER TABLE role_permissions
    ADD CONSTRAINT fk_role_permissions_role_id FOREIGN KEY (role_id)
        REFERENCES roles (id) ON DELETE RESTRICT ON UPDATE CASCADE;
ALTER TABLE role_permissions
    ADD CONSTRAINT fk_role_permissions_permission_id FOREIGN KEY (permission_id)
        REFERENCES permissions (id) ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey user_roles table
ALTER TABLE user_roles
    ADD CONSTRAINT fk_user_roles_user_id FOREIGN KEY (user_id)
        REFERENCES users (id) ON DELETE RESTRICT ON UPDATE CASCADE;
ALTER TABLE user_roles
    ADD CONSTRAINT fk_user_roles_role_id FOREIGN KEY (role_id)
        REFERENCES roles (id) ON DELETE RESTRICT ON UPDATE CASCADE;

--- AddForeignKey user_avatars table
ALTER TABLE user_avatars
    ADD CONSTRAINT fk_user_avatars_user_id FOREIGN KEY (user_id)
        REFERENCES users (id) ON DELETE RESTRICT ON UPDATE CASCADE;
ALTER TABLE user_avatars
    ADD CONSTRAINT fk_user_avatars_file_id FOREIGN KEY (file_id)
        REFERENCES files (id) ON DELETE CASCADE ON UPDATE CASCADE;

--- AddForeignKey warehouse_zones table
ALTER TABLE warehouse_zones
    ADD CONSTRAINT fk_warehouse_zones_location_id FOREIGN KEY (location_id)
        REFERENCES locations (id) ON DELETE RESTRICT;

--- AddForeignKey storage_bins table
ALTER TABLE storage_bins
    ADD CONSTRAINT fk_storage_bins_zone_id FOREIGN KEY (zone_id)
        REFERENCES warehouse_zones (id) ON DELETE RESTRICT;

--- AddForeignKey vendor_items table
ALTER TABLE vendor_items
    ADD CONSTRAINT fk_vendor_items_vendor FOREIGN KEY (vendor_id)
        REFERENCES vendors (id) ON DELETE RESTRICT;
ALTER TABLE vendor_items
    ADD CONSTRAINT fk_vendor_items_item FOREIGN KEY (item_id)
        REFERENCES items (id) ON DELETE RESTRICT;

 --- AddForeignKey item_uom_conversions table
ALTER TABLE item_uom_conversions
    ADD CONSTRAINT fk_item_uom_conversions_item FOREIGN KEY (item_id)
        REFERENCES items (id) ON DELETE RESTRICT;
