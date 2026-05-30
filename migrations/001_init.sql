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
-- timezone = 'UTC';

-- create refresh_tokens table
CREATE TABLE refresh_tokens
(
    id         UUID        NOT NULL DEFAULT uuidv7(),
    user_id    UUID        NOT NULL,
    token_hash TEXT        NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked    BOOLEAN     NOT NULL DEFAULT FALSE,
    user_agent TEXT,
    ip_address TEXT,
    CONSTRAINT pk_refresh_tokens PRIMARY KEY (id)
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
    id             UUID           NOT NULL DEFAULT uuidv7(),
    email          VARCHAR(255)   NOT NULL,
    password_hash  TEXT           ,
    username       VARCHAR(100)   ,
    status         VARCHAR(20)    NOT NULL DEFAULT 'ACTIVE', -- ACTIVE | DEACTIVATED | PENDING_PASSWORD
    deactivated_at TIMESTAMPTZ(3),                           -- vô hiệu hoá lúc nào
    created_at     TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_users PRIMARY KEY (id)
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


CREATE TABLE IF NOT EXISTS password_setup_tokens (
    id          UUID            NOT NULL DEFAULT uuidv7(),
    user_id     UUID            NOT NULL,
    token_hash  CHAR(64)        NOT NULL,
    expires_at  TIMESTAMPTZ(3)  NOT NULL,
    used_at     TIMESTAMPTZ(3),
    created_at  TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_password_setup_tokens PRIMARY KEY (id),
    CONSTRAINT uq_password_setup_tokens_token_hash UNIQUE (token_hash),
    CONSTRAINT fk_password_setup_tokens_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_password_setup_tokens_user_id ON password_setup_tokens(user_id);
CREATE INDEX idx_password_setup_tokens_expires_at ON password_setup_tokens(expires_at);



-- ==========================================================
-- Index
-- ==========================================================

-- create refresh_tokens index
CREATE INDEX idx_refresh_user ON refresh_tokens (user_id);
CREATE INDEX idx_token_hash ON refresh_tokens (token_hash);
CREATE INDEX idx_user_revoked ON refresh_tokens (user_id, revoked);

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
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email_unique ON users (email);

-- create user_avatars index
CREATE INDEX IF NOT EXISTS idx_user_avatars_selected ON user_avatars (is_primary) WHERE is_primary IS TRUE;


-- ==========================================================
-- Khoá ngoại
-- ==========================================================

--- AddForeignKey refresh_tokens table
ALTER TABLE refresh_tokens
    ADD CONSTRAINT fk_refresh_tokens_user_id FOREIGN KEY (user_id)
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

-- ==========================================================
-- Trigger
-- ==========================================================


