# Build stage
FROM rust:1.84.0-bookworm AS builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY shared shared
COPY services/progress_command_service services/progress_command_service

# Build the service
RUN cargo build --release --package progress_command_service

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/progress_command_service /app/progress_command_service

EXPOSE 8090

CMD ["./progress_command_service"]