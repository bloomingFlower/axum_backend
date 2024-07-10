-- Create User Table
CREATE TABLE IF NOT EXISTS "user" (
    id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,
    username VARCHAR(128) NOT NULL UNIQUE,
    password VARCHAR(256),
    password_salt uuid NOT NULL DEFAULT gen_random_uuid(),
    token_salt uuid NOT NULL DEFAULT gen_random_uuid()
);

-- Create Task Table
CREATE TABLE IF NOT EXISTS task (
    id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,
    title VARCHAR(256) NOT NULL,
    done bool NOT NULL DEFAULT FALSE
);