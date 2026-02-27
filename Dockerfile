# Dockerfile

FROM rust:latest AS builder
WORKDIR /app
COPY . .
# Install deps for openssl-sys + sqlx-cli
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo install sqlx-cli --no-default-features --features postgres,runtime-tokio --version 0.8.6
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
