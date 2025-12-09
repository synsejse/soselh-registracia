CREATE TABLE voting_sessions (
    session_token VARCHAR(64) PRIMARY KEY,
    display_name VARCHAR(100) NOT NULL,
    ip_address VARCHAR(45),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    voter_id VARCHAR(5) NOT NULL DEFAULT '00000'
);

CREATE TABLE candidates (
    id INTEGER AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(150) NOT NULL
);

CREATE TABLE votes (
    id INTEGER AUTO_INCREMENT PRIMARY KEY,
    session_token VARCHAR(64) NOT NULL,
    candidate_id INTEGER NOT NULL,
    voted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT unique_vote_per_user UNIQUE (session_token),
    FOREIGN KEY (session_token) REFERENCES voting_sessions(session_token),
    FOREIGN KEY (candidate_id) REFERENCES candidates(id)
);

CREATE TABLE settings (
    key_name VARCHAR(50) PRIMARY KEY,
    value VARCHAR(255) NOT NULL
);

INSERT INTO settings (key_name, value) VALUES ('voting_enabled', 'false');
