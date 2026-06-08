COPY users (id, email, password_hash, username, status)
    FROM '/tmp/users.csv' WITH (FORMAT csv, HEADER true, DELIMITER ',');

COPY roles (id, name, description, status, can_delete, can_update)
    FROM '/tmp/roles.csv' WITH (FORMAT csv, HEADER true, DELIMITER ',');

COPY permissions (id, code, description)
    FROM '/tmp/permissions.csv' WITH (FORMAT csv, HEADER true, DELIMITER ',');

COPY role_permissions (role_id, permission_id)
    FROM '/tmp/role_permissions.csv' WITH (FORMAT csv, HEADER true, DELIMITER ',');

COPY user_roles (user_id, role_id)
    FROM '/tmp/user_roles.csv' WITH (FORMAT csv, HEADER true, DELIMITER ',');

-- COPY locations (id, name, address)
--     FROM '/tmp/locations.csv' WITH (FORMAT csv, HEADER true, DELIMITER ',');

-- COPY warehouse_zones (id, location_id, name, zone_type, deleted_at)
--     FROM '/tmp/warehouse_zones.csv' WITH (FORMAT csv, HEADER true, DELIMITER ',');

-- COPY storage_bins (id, zone_id, label_name, deleted_at)
--     FROM '/tmp/storage_bins.csv' WITH (FORMAT csv, HEADER true, DELIMITER ',');

-- COPY items (id, sku, name, type, deleted_at)
--     FROM '/tmp/items.csv' WITH (FORMAT csv, HEADER true, DELIMITER ',');