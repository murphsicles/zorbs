# Dockerfile
FROM rust:bookworm AS builder
WORKDIR /app
COPY . .
COPY .sqlx .sqlx
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl wget && rm -rf /var/lib/apt/lists/* && wget -q https://dl.min.io/client/mc/release/linux-amd64/mc -O /usr/local/bin/mc && chmod +x /usr/local/bin/mc && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zorbs /usr/local/bin/zorbs
WORKDIR /app
RUN mkdir -p /uploads
EXPOSE 3000
CMD ["zorbs"]
