CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    token TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP DEFAULT (datetime('now', '+6 months')),

    FOREIGN KEY(user_id) REFERENCES users(id)
);

-- Index to query sessions by user_id.
CREATE INDEX idx_sessions_user_id ON sessions(user_id);

-- Index to query sessions by token.
CREATE INDEX idx_sessions_token ON sessions(token);
