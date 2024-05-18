-- DEV ONLY - Brute force DROP DB (for local dev and unit test)
SELECT pg_terminate_backend(pg_stat_activity.pid)
FROM pg_stat_activity
WHERE pg_stat_activity.datname = 'dev_app';
DROP DATABASE IF EXISTS dev_app;
DROP USER IF EXISTS dev_app;

-- DEV ONLY - Dev only password (for local dev and unit test)
CREATE USER dev_app PASSWORD 'dev_app';
CREATE DATABASE dev_app owner dev_app ENCODING = 'UTF8';