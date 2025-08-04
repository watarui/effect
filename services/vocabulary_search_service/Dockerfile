# Build stage
FROM rust:1.84-slim AS builder

# Install dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY services/vocabulary_search_service ./services/vocabulary_search_service

# Build the service
WORKDIR /app/services/vocabulary_search_service
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 app

# Copy binary from builder
COPY --from=builder /app/target/release/vocabulary_search_service /usr/local/bin/vocabulary_search_service

# Change ownership
RUN chown app:app /usr/local/bin/vocabulary_search_service

# Switch to non-root user
USER app

# Expose gRPC port
EXPOSE 50053

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/bin/nc", "-z", "localhost", "50053"]

# Run the service
CMD ["vocabulary_search_service"]