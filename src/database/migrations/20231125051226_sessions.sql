CREATE TABLE sessions (
    session_id bytea PRIMARY KEY,
    user_id bytea, 
    token VARCHAR(255) NOT NULL,
    issued_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    expire_at TIMESTAMPTZ,
    ip_address VARCHAR(45),
    user_agent VARCHAR(255),
    status BOOLEAN DEFAULT true,
    FOREIGN KEY (user_id) REFERENCES accounts(id)
);