CREATE TABLE IF NOT EXISTS accounts (
    id BYTEA PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,
    password VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    code_verification VARCHAR(255) NOT NULL,
    email_verified_at TIMESTAMPTZ
);