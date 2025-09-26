FROM rust:1.90.0-slim-bookworm AS builder

WORKDIR /app
COPY ./ /app

# In order to build the project on Linux we need these dependencies.
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Install sqlx-cli to create the initial database.
RUN cargo install sqlx-cli --no-default-features --features sqlite

# Set SQLX to offline mode to avoid needing a live database during build.
COPY .sqlx ./.sqlx
ENV SQLX_OFFLINE=true

RUN cargo build --release
RUN strip target/release/mrkt

FROM debian:bookworm-20250908-slim

# Install runtime dependencies.
RUN apt-get update && apt-get install -y libssl3 ca-certificates

# Create the app directory structure.
RUN mkdir -p /app
WORKDIR /app

# Copy the binary, static, configuration and migration files.
COPY --from=builder /app/Rocket.toml .
COPY --from=builder /app/target/release/mrkt .
COPY --from=builder /app/static ./static
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/
COPY --from=builder /app/migrations ./migrations


# Create data directory for the database.
RUN mkdir -p /app/data

# Copy and setup startup script.
COPY startup.sh /start.sh
RUN chmod +x /start.sh

EXPOSE 8000

ENTRYPOINT ["/start.sh"]