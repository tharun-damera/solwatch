# Stage 1: Build the binary
FROM rust:1.91.0-slim-bookworm AS builder
WORKDIR /app

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./src ./src

RUN cargo build --release

# Stage 2: Minimal Runtime Image
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/backend .
EXPOSE 5000

ENTRYPOINT ["./backend"]
