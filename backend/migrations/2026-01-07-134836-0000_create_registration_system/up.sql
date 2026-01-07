-- Create sessions table for field trips (odborové dni)
CREATE TABLE sessions (
    id INTEGER AUTO_INCREMENT PRIMARY KEY,
    field_code VARCHAR(3) NOT NULL,
    field_name VARCHAR(100) NOT NULL,
    session_date DATE NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    max_capacity INTEGER NOT NULL,
    turnus INTEGER NOT NULL,
    INDEX idx_field_code (field_code),
    INDEX idx_session_date (session_date)
);

-- Create registrations table
CREATE TABLE registrations (
    id INTEGER AUTO_INCREMENT PRIMARY KEY,
    session_id INTEGER NOT NULL,
    student_first_name VARCHAR(100) NOT NULL,
    student_last_name VARCHAR(100) NOT NULL,
    guardian_first_name VARCHAR(100) NOT NULL,
    guardian_last_name VARCHAR(100) NOT NULL,
    guardian_phone VARCHAR(20) NOT NULL,
    guardian_email VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (session_id) REFERENCES sessions(id),
    INDEX idx_session_id (session_id)
);

-- Create settings table for system configuration
CREATE TABLE settings (
    key_name VARCHAR(50) PRIMARY KEY,
    value VARCHAR(255) NOT NULL
);

-- Insert settings
INSERT INTO settings (key_name, value) VALUES ('registration_enabled', 'true');

-- Insert session data for Turnus 1 (26.1.2026-30.1.2026)
INSERT INTO sessions (field_code, field_name, session_date, start_time, end_time, max_capacity, turnus) VALUES
('ELE', 'Elektrotechnika', '2026-01-26', '08:00:00', '09:30:00', 10, 1),
('MUM', 'Multimédia', '2026-01-28', '08:00:00', '10:30:00', 10, 1),
('MPS', 'Mechanik počítačových sietí', '2026-01-30', '08:00:00', '09:30:00', 10, 1),
('IST', 'Informačné a sieťové technológie', '2026-01-30', '09:45:00', '12:10:00', 10, 1);

-- Insert session data for Turnus 2 (3.2.2026-9.2.2026)
INSERT INTO sessions (field_code, field_name, session_date, start_time, end_time, max_capacity, turnus) VALUES
('MUM', 'Multimédia', '2026-02-04', '08:00:00', '10:30:00', 10, 2),
('MPS', 'Mechanik počítačových sietí', '2026-02-06', '08:00:00', '09:30:00', 10, 2),
('IST', 'Informačné a sieťové technológie', '2026-02-06', '09:45:00', '12:10:00', 10, 2),
('ELE', 'Elektrotechnika', '2026-02-09', '08:00:00', '09:30:00', 10, 2);
