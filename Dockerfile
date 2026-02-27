# Dockerfile

FROM rust:latest AS builder
WORKDIR /app
COPY . .

# Install system deps for openssl-sys and sqlx-cli
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Install sqlx-cli (correct features - no runtime-tokio for CLI)
RUN cargo install sqlx-cli --no-default-features --features postgres

# Prepare sqlx queries (must run BEFORE SQLX_OFFLINE=true)
RUN cargo sqlx prepare -- --bin zorbs

ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zorbs /usr/local/bin/zorbs
WORKDIR /app
RUN mkdir -p /uploads
EXPOSE 3000
CMD ["zorbs"]
