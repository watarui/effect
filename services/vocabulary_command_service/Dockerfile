# Build stage
FROM rust:1.84.0-bookworm AS builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY shared shared
COPY services/vocabulary_command_service services/vocabulary_command_service

# Build the service
RUN cargo build --release --package vocabulary_command_service

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/vocabulary_command_service /app/vocabulary_command_service

EXPOSE 8080

CMD ["./vocabulary_command_service"]