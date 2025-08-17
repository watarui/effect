# Build stage
FROM rust:1.89 AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY shared shared
COPY services services
COPY protos protos
COPY services/vocabulary_search_service services/vocabulary_search_service

# Build the service
RUN cargo build --release --package vocabulary_search_service

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/vocabulary_search_service /app/vocabulary_search_service

EXPOSE 8084

CMD ["./vocabulary_search_service"]