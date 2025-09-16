# mrkt

A self-hosted portfolio tracker.

## 🛠️ Local build

In order to have a local build of the project, you need to have a few dependencies installed:

- [Rust](https://rustup.rs/), which is the central piece used to build the project.
- [sqlx-cli](https://docs.rs/crate/sqlx-cli/latest), which is used to manage the SQLite database.
- [Node.js](https://nodejs.org/en/), which is used to build trigger builds for the styling (Tailwind CSS).

Once you have all the dependencies ready, clone the repository and run:

```bash
npm i                        # Install Tailwind and its dependencies.
cp .env.local.example .env   # Setup environment variables from the example file.
sqlx database create         # Create the actual DB file.
sqlx migrate run             # Run all pending migrations.
```

> [!TIP]
> You can also modify the `.env` file to change the database location if you want to store it somewhere else,
> by default it's set to `mrkt.db` in the `target` directory.

Once you have the dependencies and database set up, you can start the server by running:

```bash
cargo run

# Or, if you have cargo-watch installed
cargo watch -x run # Automatically rebuilds and restarts the server on code changes.
```
