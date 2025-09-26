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

Before continuing, copy the `.env.local` file to `.env` and set up at least the `OPENEXCHANGERATES_APP_ID`
to be able to pull currency exchange rates. See the example environment file for more details.

Once you have the dependencies and database set up, you can start the server by running:

```bash
cargo run

# Or, if you have cargo-watch installed
cargo watch -x run # Automatically rebuilds and restarts the server on code changes.
```

## 🐳 Docker Deployment

Pre-built Docker images are automatically published to GitHub Container Registry on every release. These images support both AMD64 and ARM64 architectures.

### Using Docker Compose

Copy over the repository's `docker-compose.yml` file and follow `.env.docker` to create your own `.env`
file with the necessary environment variables. If you want to create your own compose file, make sure
that you use the `ghcr.io/sleepyfran/mrkt:latest` image and mount a volume to persist the database.

Then, you can run:

```bash
docker-compose up -d
```

And the server will be available at `http://localhost:8000`. Mrkt uses Rocket, which technically
can provide TLS support, but it's recommended to use a reverse proxy like Nginx or Caddy to handle it.
