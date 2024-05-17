-- DEV ONLY - Brute force DROP DB (for local dev and unit test)
SELECT pg_terminate_backend(pg_stat_activity.pid)
FROM pg_stat_activity
WHERE pg_stat_activity.datname = 'mydb';
DROP DATABASE IF EXISTS mydb;
DROP USER IF EXISTS myuser;

-- DEV ONLY - Dev only password (for local dev and unit test)
CREATE USER myuser PASSWORD 'myuser_pwd';
CREATE DATABASE mydb owner myuser ENCODING = 'UTF8';