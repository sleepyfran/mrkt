CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    owner_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY(owner_id) REFERENCES users(id)
);

CREATE TABLE transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    owner_id INTEGER NOT NULL,
    account_id INTEGER NOT NULL,
    transaction_type INTEGER NOT NULL CHECK (transaction_type IN (0, 1)),
    ticker_symbol TEXT NOT NULL,
    transaction_date TEXT NOT NULL,
    quantity REAL NOT NULL CHECK (quantity > 0),
    price_per_share REAL NOT NULL CHECK (price_per_share > 0),
    fees REAL NOT NULL CHECK (fees >= 0),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY(owner_id) REFERENCES users(id)
    FOREIGN KEY(account_id) REFERENCES accounts(id)
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
