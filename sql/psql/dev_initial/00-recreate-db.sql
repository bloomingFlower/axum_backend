-- DEV ONLY - Brute force DROP DB (for local dev and unit test)
-- Terminate all connections to the database
SELECT pg_terminate_backend(pg_stat_activity.pid)
FROM pg_stat_activity
WHERE pg_stat_activity.datname = 'dev_app';
-- Drop the database
DROP DATABASE IF EXISTS dev_app;
-- Drop the user
DROP USER IF EXISTS dev_app;
-- DEV ONLY - Dev only password (for local dev and unit test)
CREATE USER dev_app PASSWORD 'dev_app';
-- DEV ONLY - Create the database (for local dev and unit test)
CREATE DATABASE dev_app owner dev_app ENCODING = 'UTF8';