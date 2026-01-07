-- Create admin sessions table for authentication
CREATE TABLE admin_sessions (
    session_token VARCHAR(36) PRIMARY KEY,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NULL,
    ip_address VARCHAR(45) NULL,
    INDEX idx_expires_at (expires_at)
);
