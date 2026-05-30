-- -- ==========================================================
-- -- 1. QUẢN LÝ KHO (WAREHOUSE STRUCTURE)
-- -- ==========================================================
-- CREATE TABLE locations
-- (
--     id         UUID           NOT NULL DEFAULT uuidv7(),
--     name       VARCHAR(150)   NOT NULL, -- Kho Quận 9
--     address    VARCHAR(255),
--     deleted_at TIMESTAMPTZ(3),          -- xoá mềm
--     created_at TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
--     updated_at TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
--     CONSTRAINT pk_locations PRIMARY KEY (id)
-- );

-- CREATE TABLE warehouse_zones
-- (
--     id          UUID         NOT NULL DEFAULT uuidv7(),
--     location_id UUID         NOT NULL,
--     name        VARCHAR(150) NOT NULL, -- Kho Nguyên Liệu - Khu A
--     zone_type   VARCHAR(50)  NOT NULL, -- Loại: FINISHED_GOODS, RAW_MATERIAL, PACKAGING, SOLVENT
--     deleted_at  TIMESTAMPTZ(3),        -- xoá mềm
--     CONSTRAINT pk_warehouse_zones PRIMARY KEY (id)
-- );

-- CREATE TABLE storage_bins
-- (
--     id         UUID         NOT NULL DEFAULT uuidv7(),
--     zone_id    UUID         NOT NULL,
--     label_name VARCHAR(255) NOT NULL, -- Nhãn: Tạm Trữ, Bảo quản, Loại bỏ, Hàng trả về
--     deleted_at TIMESTAMPTZ(3),        -- xoá mềm
--     CONSTRAINT unique_storage_bins UNIQUE (zone_id, label_name),
--     CONSTRAINT pk_storage_bins PRIMARY KEY (id)
-- );


-- -- ==========================================================
-- -- 2. DANH MỤC VẬT TƯ (ITEM MASTER)
-- -- ==========================================================
-- CREATE TABLE IF NOT EXISTS items
-- (
--     id         UUID           NOT NULL DEFAULT uuidv7(),
--     sku        VARCHAR(50),             -- mã nguyên liệu nội bộ
--     name       VARCHAR(255)   NOT NULL, -- tên nguyên liệu COA | tên bao bì | tên util
--     type       VARCHAR(50)    NOT NULL, -- loại item PACKAGING | RAW_MATERIAL | UTILITY
--     deleted_at TIMESTAMPTZ(3),          -- xoá mềm
--     created_at TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
--     updated_at TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
--     CONSTRAINT pk_items PRIMARY KEY (id),
--     CONSTRAINT items_sku_unique UNIQUE (sku)
-- );

-- CREATE TABLE IF NOT EXISTS item_uom_conversions
-- (
--     id                UUID                    DEFAULT uuidv7(),
--     item_id           UUID           NOT NULL,

--     -- Tên đơn vị quy đổi (Ví dụ: 'Phuy 250kg', 'Thùng', 'Bao')
--     alternative_uom   VARCHAR(100)   NOT NULL,
--     -- Tỷ lệ nhân để ra Đơn vị cơ sở (Ví dụ: 1 Phuy = 250 kg -> conversion_factor = 250)
--     conversion_factor DECIMAL(15, 6) NOT NULL CHECK (conversion_factor > 0),
--     -- (Tùy chọn) Đánh dấu xem đơn vị này có được phép dùng để mua hàng không
--     is_purchase_uom   BOOLEAN                 DEFAULT true,
--     is_base           BOOLEAN        NOT NULL,

--     deleted_at        TIMESTAMPTZ(3), -- xoá mềm
--     created_at        TIMESTAMPTZ(3)          DEFAULT NOW(),
--     updated_at        TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
--     CONSTRAINT pk_item_uom_conversions PRIMARY KEY (id),
--     CONSTRAINT chk_item_uom_conversions_base_factor CHECK (NOT is_base OR conversion_factor = 1)

-- );
-- -- Mỗi item chỉ có đúng 1 base UOM (loại trừ soft-deleted)
-- CREATE UNIQUE INDEX uix_item_uom_base
--     ON item_uom_conversions (item_id)
--     WHERE (is_base = TRUE AND deleted_at IS NULL);
-- -- Tránh trùng tên đơn vị trong cùng một item
-- CREATE UNIQUE INDEX uix_item_uom_name
--     ON item_uom_conversions (item_id, alternative_uom)
--     WHERE (deleted_at IS NULL);


-- -- ==========================================================
-- -- 2. QUẢN LÝ LÔ VÀ KIỂM NGHIỆM (LOTS & QC)
-- -- ==========================================================
-- CREATE TABLE IF NOT EXISTS item_lots
-- (
--     id               UUID NOT NULL DEFAULT uuidv7(),
--     item_id          UUID NOT NULL,
--     -- Mã hệ thống tự sinh để truy xuất nguồn gốc
--     internal_lot_no  VARCHAR(100),
--     -- Mã in trên bao bì của nhà cung cấp (nếu có)
--     vendor_lot_no    VARCHAR(100),
--     manufacture_date TIMESTAMPTZ(3),
--     expiration_date  TIMESTAMPTZ(3),
--     deleted_at       TIMESTAMPTZ(3), -- xoá mềm
--     created_at       TIMESTAMPTZ   DEFAULT NOW(),
--     updated_at       TIMESTAMPTZ   DEFAULT NOW(),
--     CONSTRAINT pk_item_lots PRIMARY KEY (id),
--     CONSTRAINT item_lots_internal_lot_no_unique UNIQUE (internal_lot_no)
-- );

-- CREATE TABLE IF NOT EXISTS quality_inspections
-- (
--     id              UUID  DEFAULT uuidv7(),
--     lot_id          UUID        NOT NULL,
--     inspection_type VARCHAR(50) NOT NULL, -- INCOMING, PRODUCTION
--     inspector_id    UUID        NOT NULL,
--     actual_metrics  JSONB,                -- Lưu kết quả pH, nồng độ, độ tinh khiết...
--     status          VARCHAR(20) NOT NULL, -- PASS, FAIL
--     deleted_at      TIMESTAMPTZ(3),       -- xoá mềm
--     created_at      TIMESTAMPTZ      DEFAULT NOW(),
--     updated_at      TIMESTAMPTZ      DEFAULT NOW(),
--     CONSTRAINT pk_quality_inspections PRIMARY KEY (id)
-- );


-- -- ==========================================================
-- -- 3. QUẢN LÝ TỒN KHO VÀ BIẾN ĐỘNG (INVENTORY)
-- -- ==========================================================

-- CREATE TABLE item_packages
-- (
--     id                UUID                    DEFAULT uuidv7(),
--     lot_id            UUID           NOT NULL REFERENCES item_lots (id),
--     package_code      VARCHAR(100)   NOT NULL,
--     parent_package_id UUID REFERENCES item_packages (id),
--     initial_qty       DECIMAL(15, 3) NOT NULL CHECK (initial_qty > 0),
--     current_qty       DECIMAL(15, 3) NOT NULL CHECK (current_qty >= 0),
--     storage_bin_id    UUID REFERENCES storage_bins (id),
--     status            VARCHAR(20)    NOT NULL DEFAULT 'ACTIVE'
--         CHECK (status IN ('ACTIVE', 'DEPLETED', 'SPLIT')),
--     notes             TEXT,
--     created_at        TIMESTAMPTZ    NOT NULL DEFAULT NOW(),
--     updated_at        TIMESTAMPTZ    NOT NULL DEFAULT NOW(),
--     CONSTRAINT pk_item_packages PRIMARY KEY (id),
--     CONSTRAINT uq_package_code UNIQUE (package_code),
--     CONSTRAINT chk_depleted_qty CHECK (
--         NOT (status = 'DEPLETED' AND current_qty > 0)
--         )
-- );

-- CREATE TABLE IF NOT EXISTS inventory_stocks
-- (
--     id             UUID                    DEFAULT uuidv7(),
--     item_id        UUID           NOT NULL,
--     lot_id         UUID           NOT NULL,
--     storage_bin_id UUID           NOT NULL,
--     quantity       DECIMAL(15, 3) NOT NULL DEFAULT 0 CHECK (quantity >= 0),
--     created_at     TIMESTAMPTZ    NOT NULL DEFAULT NOW(),
--     updated_at     TIMESTAMPTZ    NOT NULL DEFAULT NOW(),
--     CONSTRAINT pk_inventory_stocks PRIMARY KEY (id),
--     CONSTRAINT uq_inventory_stocks_item_lot_bin UNIQUE (item_id, lot_id, storage_bin_id)
-- );

-- -- CREATE TABLE IF NOT EXISTS inventory_transactions
-- -- (
-- --     id                UUID PRIMARY KEY DEFAULT uuidv7(),
-- --     item_id           UUID           NOT NULL,
-- --     lot_id            UUID,
-- --     storage_bin_id    UUID           NOT NULL,
-- --     transaction_type  VARCHAR(50)    NOT NULL, -- PURCHASE_RECEIPT, PRODUCTION_ISSUE, PRODUCTION_RECEIPT, INTERNAL_TRANSFER
-- --     package_id        UUID           NOT NULL,
-- --     source_package_id UUID,                    -- Nếu là +18kg vào thùng, thì đây là ID của Phuy mẹ
-- --     reason_code       VARCHAR(50),             -- Ví dụ: 'REFILL', 'INITIAL_DISPENSE'
-- --     quantity          DECIMAL(15, 3) NOT NULL, -- Dương: tăng, Âm: giảm
-- --     reference_id      UUID,                    -- ID của Receipt_Item hoặc Production_Item
-- --     user_id           UUID           NOT NULL,
-- --     created_at        TIMESTAMPTZ      DEFAULT NOW()
-- -- );

-- CREATE TABLE inventory_transactions
-- (
--     id                UUID PRIMARY KEY        DEFAULT uuidv7(),
--     item_id           UUID           NOT NULL REFERENCES items (id),
--     lot_id            UUID REFERENCES item_lots (id),
--     storage_bin_id    UUID           NOT NULL REFERENCES storage_bins (id),
--     transaction_type  VARCHAR(50)    NOT NULL CHECK (transaction_type IN (
--                                                                           'PURCHASE_RECEIPT',
--                                                                           'SPLIT_OUT',
--                                                                           'SPLIT_IN',
--                                                                           'PRODUCTION_ISSUE',
--                                                                           'PRODUCTION_RECEIPT',
--                                                                           'INTERNAL_TRANSFER',
--                                                                           'ADJUSTMENT'
--         )),
--     package_id        UUID           NOT NULL REFERENCES item_packages (id),
--     source_package_id UUID REFERENCES item_packages (id),
--     quantity          DECIMAL(15, 3) NOT NULL, -- (+) nhập  |  (−) xuất, luôn base UOM
--     reference_id      UUID,
--     reference_type    VARCHAR(50),
--     reason_code       VARCHAR(50),
--     user_id           UUID           NOT NULL,
--     created_at        TIMESTAMPTZ    NOT NULL DEFAULT NOW()
-- );


-- ALTER TABLE warehouse_zones
--     ADD CONSTRAINT fk_warehouse_zones_location_id FOREIGN KEY (location_id)
--         REFERENCES locations (id) ON DELETE RESTRICT;

-- ALTER TABLE storage_bins
--     ADD CONSTRAINT fk_storage_bins FOREIGN KEY (zone_id)
--         REFERENCES warehouse_zones (id) ON DELETE RESTRICT;

-- ALTER TABLE item_uom_conversions
--     ADD CONSTRAINT fk_item_uom_conversions_item_id FOREIGN KEY (item_id)
--         REFERENCES items (id) ON DELETE RESTRICT;

-- ALTER TABLE item_lots
--     ADD CONSTRAINT fk_item_lots_item_id FOREIGN KEY (item_id)
--         REFERENCES items (id) ON DELETE RESTRICT;

-- ALTER TABLE quality_inspections
--     ADD CONSTRAINT fk_quality_inspections_inspector_id FOREIGN KEY (inspector_id)
--         REFERENCES users (id) ON DELETE RESTRICT;

-- ALTER TABLE quality_inspections
--     ADD CONSTRAINT fk_quality_inspections_lot_id FOREIGN KEY (lot_id)
--         REFERENCES item_lots (id) ON DELETE RESTRICT;

