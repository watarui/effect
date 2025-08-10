# Build stage
FROM rust:1.83 AS builder

WORKDIR /usr/src/app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY services/event_store_service ./services/event_store_service
COPY protos ./protos

# Build the service
RUN cargo build --release -p event_store_service

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/event_store_service /app/

# Copy migrations
COPY services/event_store_service/migrations ./migrations

# Expose gRPC port
EXPOSE 50051

CMD ["./event_store_service"]