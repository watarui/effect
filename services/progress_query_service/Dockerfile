FROM rust:1.89 AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Build dependencies
COPY Cargo.toml Cargo.lock ./
COPY shared shared
COPY services services
COPY protos protos

# Build the service
RUN cargo build --release -p progress_query_service

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