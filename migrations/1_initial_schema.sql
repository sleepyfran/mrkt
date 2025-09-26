-- ================== USERS.
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    hashed_password TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ================== SESSIONS.
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    token TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL DEFAULT (datetime('now', '+6 months')),

    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Index to query sessions by user_id.
CREATE INDEX idx_sessions_user_id ON sessions(user_id);

-- Index to query sessions by token.
CREATE INDEX idx_sessions_token ON sessions(token);


-- ================== ACCOUNTS.
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    owner_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY(owner_id) REFERENCES users(id) ON DELETE CASCADE
);

-- ================== TRANSACTIONS.
CREATE TABLE transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    owner_id INTEGER NOT NULL,
    account_id INTEGER NOT NULL,
    transaction_type INTEGER NOT NULL CHECK (transaction_type IN (0, 1, 2)),
    ticker_symbol TEXT NOT NULL,
    transaction_date TEXT NOT NULL,
    quantity REAL NOT NULL CHECK (quantity > 0),
    price_per_share REAL NOT NULL CHECK (price_per_share >= 0),
    currency TEXT NOT NULL,
    fees REAL NOT NULL CHECK (fees >= 0),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY(owner_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

-- Index for querying by owner_id.
CREATE INDEX idx_transactions_owner_id ON transactions(owner_id);

-- Index for querying by account_id.
CREATE INDEX idx_transactions_account_id ON transactions(account_id);

-- Index for querying by ticker symbol.
CREATE INDEX idx_transactions_ticker_symbol ON transactions(ticker_symbol);

-- Index for querying by date.
CREATE INDEX idx_transactions_date ON transactions(transaction_date);

-- Composite index for ticker and date queries.
CREATE INDEX idx_transactions_ticker_date ON transactions(ticker_symbol, transaction_date);
