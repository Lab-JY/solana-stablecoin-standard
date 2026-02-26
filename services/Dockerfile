# Multi-stage Rust build for all SSS services
# Usage: docker build --build-arg SERVICE=mint-burn -t sss-mint-burn .

FROM rust:1.82-bookworm AS builder

ARG SERVICE

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY programs/sss-token/Cargo.toml programs/sss-token/Cargo.toml
COPY programs/sss-transfer-hook/Cargo.toml programs/sss-transfer-hook/Cargo.toml
COPY services/shared/Cargo.toml services/shared/Cargo.toml
COPY services/mint-burn/Cargo.toml services/mint-burn/Cargo.toml
COPY services/compliance/Cargo.toml services/compliance/Cargo.toml
COPY services/indexer/Cargo.toml services/indexer/Cargo.toml
COPY services/webhook/Cargo.toml services/webhook/Cargo.toml

# Create dummy source files to cache dependencies
RUN mkdir -p programs/sss-token/src && echo "fn main() {}" > programs/sss-token/src/lib.rs && \
    mkdir -p programs/sss-transfer-hook/src && echo "fn main() {}" > programs/sss-transfer-hook/src/lib.rs && \
    mkdir -p services/shared/src && echo "" > services/shared/src/lib.rs && \
    mkdir -p services/mint-burn/src && echo "fn main() {}" > services/mint-burn/src/main.rs && \
    mkdir -p services/compliance/src && echo "fn main() {}" > services/compliance/src/main.rs && \
    mkdir -p services/indexer/src && echo "fn main() {}" > services/indexer/src/main.rs && \
    mkdir -p services/webhook/src && echo "fn main() {}" > services/webhook/src/main.rs

# Build dependencies only (cached layer)
RUN cargo build --release -p sss-${SERVICE} 2>/dev/null || true

# Copy actual source code
COPY programs/ programs/
COPY services/ services/

# Touch source files to invalidate cache
RUN find services/ -name "*.rs" -exec touch {} +

# Build the target service
RUN cargo build --release -p sss-${SERVICE}

# Runtime stage
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

ARG SERVICE

# Create non-root user
RUN useradd -m -s /bin/bash sss

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/sss-${SERVICE} /app/sss-service

# Create data directory for SQLite
RUN mkdir -p /app/data && chown -R sss:sss /app

USER sss

ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:/app/data/sss.db

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["/app/sss-service"]
