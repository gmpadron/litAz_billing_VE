# ============================================================
# Stage 1: Build
# ============================================================
FROM rust:1.94.1-bookworm AS builder

WORKDIR /app

# Copiar manifests primero para cachear dependencias
COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml migration/Cargo.toml

# Crear src dummy para compilar solo dependencias
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    mkdir -p migration/src && echo "" > migration/src/lib.rs

RUN cargo build --release 2>/dev/null || true

# Copiar código real y compilar
RUN rm -rf src migration/src
COPY src/ src/
COPY migration/src/ migration/src/

# Forzar recompilación del binario (no de las deps)
RUN touch src/main.rs migration/src/lib.rs && cargo build --release

# ============================================================
# Stage 2: Runtime
# ============================================================
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/billing_core /app/billing_core

EXPOSE 8080

CMD ["/app/billing_core"]
