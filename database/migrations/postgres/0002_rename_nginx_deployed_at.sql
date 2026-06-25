-- Rename typo column Webed_at -> deployed_at on web_nginx_config

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'web_nginx_config'
          AND column_name = 'webed_at'
    ) THEN
        ALTER TABLE web_nginx_config RENAME COLUMN webed_at TO deployed_at;
    ELSIF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'web_nginx_config'
          AND column_name = 'Webed_at'
    ) THEN
        ALTER TABLE web_nginx_config RENAME COLUMN "Webed_at" TO deployed_at;
    END IF;
END $$;
