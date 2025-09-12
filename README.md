# mrkt

A self-hosted portfolio tracker.

## Local build

mrkt uses [SQLite](https://www.sqlite.org/index.html) as its database, and handles
the initial setup and migrations through the [sqlx-cli](https://docs.rs/crate/sqlx-cli/latest).

To get started, start by installing the [sqlx-cli](https://docs.rs/crate/sqlx-cli/latest) tool
and running the following commands:

```bash
sqlx create database # Create the actual db file
sqlx migrate run     # Run all pending migrations
```

Once you have the database set up, you can start the API server by running:

```bash
cargo run
```
