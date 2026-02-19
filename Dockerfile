# Dockerfile

FROM rust:1.82 as builder

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/zorbs /usr/local/bin/zorbs

WORKDIR /app
RUN mkdir -p /uploads

EXPOSE 3000

CMD ["zorbs"]
