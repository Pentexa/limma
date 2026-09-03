# Çok basit multi-stage Dockerfile - geliştirici/CI için başlangıç
# NOT: Bu örnek, workspace ve bağımlılıklarınıza göre özelleştirilmeli.

# Build stage
FROM rust:1.70 as builder
WORKDIR /usr/src/limma

# Önce Cargo.toml/lock kopyalanıp bağımlılıklar cache'lenebilir (workspace yapısına göre uyarlayın)
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
COPY limma-shared ./limma-shared

WORKDIR /usr/src/limma/backend
# Release derlemesi
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Builder'dan ikili dosyayı kopyala (binary adı: limma)
COPY --from=builder /usr/src/limma/backend/target/release/limma /usr/local/bin/limma

ENV RUST_LOG=info
EXPOSE 8900
CMD ["/usr/local/bin/limma"]
