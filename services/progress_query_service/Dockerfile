FROM rust:1.84-slim AS builder

WORKDIR /app

# Build dependencies
COPY Cargo.toml Cargo.lock ./
COPY services/progress_query_service/Cargo.toml services/progress_query_service/

# Create dummy source for dependency compilation
RUN mkdir -p services/progress_query_service/src && \
    echo "fn main() {}" > services/progress_query_service/src/main.rs

# Build dependencies
RUN cargo build --release -p progress_query_service

# Build application
COPY services/progress_query_service/src services/progress_query_service/src
RUN touch services/progress_query_service/src/main.rs && \
    cargo build --release -p progress_query_service

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/progress_query_service /app/
COPY services/progress_query_service/migrations migrations/

EXPOSE 8091

CMD ["./progress_query_service"]