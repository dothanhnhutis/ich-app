
-- Tạo các partition theo tháng (Nên dùng cronjob hoặc pg_partman để tạo tự động)
-- CREATE TABLE audit_logs_2026_04 PARTITION OF audit_logs FOR VALUES FROM ('2026-04-01') TO ('2026-05-01');
-- CREATE TABLE audit_logs_2026_05 PARTITION OF audit_logs FOR VALUES FROM ('2026-05-01') TO ('2026-06-01');

-- 1. Tạo schema cho pg_partman (khuyên dùng)
CREATE SCHEMA IF NOT EXISTS partman;

-- 2. Kích hoạt extension (Cần quyền Superuser hoặc Owner)
CREATE
EXTENSION IF NOT EXISTS pg_partman SCHEMA partman;

-- Chạy hàm này để partman khởi tạo các partition
SELECT partman.create_parent(
               p_parent_table := 'public.audit_logs', -- Tên bảng cha (Schema.Table)
               p_control := 'changed_at', -- Cột dùng để phân vùng
               p_interval := '1 day', -- Kích thước mỗi phân vùng (Tạo theo tháng)
               p_premake := 3 -- Số lượng partition tương lai cần tạo sẵn
       );


-- check partition work
-- SELECT *
-- FROM partman.show_partitions('public.audit_logs');
--
-- SELECT * FROM pg_stat_activity
-- WHERE application_name LIKE '%partman%';
--
-- SHOW shared_preload_libraries;
--
-- SELECT * FROM pg_settings
-- WHERE name LIKE 'pg_partman_bgw%';
--
-- SELECT partman.run_maintenance();
--
-- SELECT *
-- FROM pg_stat_activity
-- WHERE backend_type LIKE '%worker%';
--
--
-- SELECT tableoid::regclass, *
-- FROM audit_logs
-- ORDER BY changed_at DESC
-- LIMIT 1;