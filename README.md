# mrkt

A self-hosted portfolio tracker.

## 🛠️ Local build

In order to have a local build of the project, you need to have a few dependencies installed:

- [Rust](https://rustup.rs/), which is the central piece used to build the project.
- [sqlx-cli](https://docs.rs/crate/sqlx-cli/latest), which is used to manage the SQLite database.

Once you have all the dependencies ready, clone the repository and run:

```bash
cp .env.local.example .env   # Setup environment variables from the example file.
sqlx database create         # Create the actual DB file.
sqlx migrate run             # Run all pending migrations.
```

> [!TIP]
> You can also modify the `.env` file to change the database location if you want to store it somewhere else,
> by default it's set to `mrkt.db` in the `target` directory.

### Exchange Rate Providers

The application supports multiple exchange rate providers for currency conversion:

- **OpenExchangeRates** (Recommended): Set `OPENEXCHANGERATES_APP_ID` with your [free or paid plan App ID](https://openexchangerates.org/signup). The free tier is supported and provides 1,000 requests/month with USD as base currency.
- **AlphaVantage**: Set `ALPHAVANTAGE_API_KEY` with your [API key](https://www.alphavantage.co/support/#api-key). Free tier has more restrictive limits.

You need at least one exchange rate provider configured. If both are configured, OpenExchangeRates will be used as the primary provider with AlphaVantage as fallback.

Example `.env` file:

```bash
DATABASE_URL=sqlite:./target/mrkt.db
OPENEXCHANGERATES_APP_ID=your_app_id_here
# ALPHAVANTAGE_API_KEY=your_api_key_here  # Optional fallback
```

> [!IMPORTANT]
> Setting up an AlphaVantage key will also set it up as a fallback provider for stock market data.

Once you have the dependencies and database set up, you can start the server by running:

```bash
cargo run

# Or, if you have cargo-watch installed
cargo watch -x run # Automatically rebuilds and restarts the server on code changes.
```
