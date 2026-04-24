# Multi-stage build for minimal image size
FROM rust:latest AS builder

# Install build dependencies for static linking
RUN apt-get update && apt-get install -y \
    musl-tools \
    pkg-config \
    libssl-dev \
    && rustup target add x86_64-unknown-linux-musl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo files first for layer caching
COPY backend/Cargo.toml backend/Cargo.lock ./
# Create a dummy src to cache dependencies
RUN mkdir -p src/cli && \
    echo 'fn main() { println!("dummy"); }' > src/cli/main.rs && \
    echo 'fn main() {}' > src/main.rs && \
    touch src/cli/formatter.rs src/cli/scanner.rs src/cli/mod.rs && \
    cargo build --release --bin limma-cli --features cli \
        --target x86_64-unknown-linux-musl 2>/dev/null || true

# Copy actual source
COPY backend/src ./src
COPY backend/rules ./rules

# Build the real CLI binary (static-linked)
RUN cargo build --release --bin limma-cli --features cli \
    --target x86_64-unknown-linux-musl

# ─── Runtime stage ───────────────────────────────────────────────────────────
FROM alpine:3.20

# Install minimal runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    tzdata \
    curl \
    jq

# Create non-root user
RUN addgroup -g 1000 limma && \
    adduser -D -u 1000 -G limma limma

# Copy binary from builder
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/limma-cli /usr/local/bin/limma

# Copy entrypoint script
COPY .github/actions/limma-scan/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Switch to non-root user
USER limma

ENTRYPOINT ["/entrypoint.sh"]
