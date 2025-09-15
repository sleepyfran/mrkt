# mrkt

A self-hosted portfolio tracker.

## Local build

mrkt uses [SQLite](https://www.sqlite.org/index.html) as its database, and handles
the initial setup and migrations through the [sqlx-cli](https://docs.rs/crate/sqlx-cli/latest).

To get started, start by installing the [sqlx-cli](https://docs.rs/crate/sqlx-cli/latest) tool
and copy over the environment file from the local template:

```bash
cp .env.local.example .env
```

> You can also modify the `.env` file to change the database location if you want to store it somewhere else,
> by default it's set to `mrkt.db` in the `target` directory.

Once that's done, you can run the following commands to create the database and run any pending migrations:

```bash
sqlx database create # Create the actual db file
sqlx migrate run     # Run all pending migrations
```

Once you have the database set up, you can start the API server by running:

```bash
cargo run
```
