# Build stage
FROM rust:1.89 AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY services ./services
COPY protos ./protos

# Build the service
RUN cargo build --release -p domain_events_service

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/domain_events_service /app/

# Copy migrations
COPY services/domain_events_service/migrations ./migrations

# Expose gRPC port
EXPOSE 50053

CMD ["./domain_events_service"]