# ── Stage 1: oplint binary ───────────────────────────────────
FROM debian:bookworm-slim AS cli-installer
RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates && rm -rf /var/lib/apt/lists/*
ARG OPLINT_VERSION=v0.1.0
RUN curl -fsSL "https://github.com/kodaskills/oplint/releases/download/${OPLINT_VERSION}/oplint-linux-x86_64.tar.gz" \
    -o /tmp/oplint.tar.gz && \
    tar -xzf /tmp/oplint.tar.gz -C /usr/local/bin && \
    rm /tmp/oplint.tar.gz && chmod +x /usr/local/bin/oplint

# ── Stage 2: Build server (musl static binary) ───────────────
FROM rust:1.93-alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev
RUN rustup target add x86_64-unknown-linux-musl
ENV RUSTFLAGS="-C target-feature=+crt-static"
WORKDIR /app
COPY server/Cargo.toml server/Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo_registry \
    --mount=type=cache,target=/app/target,id=cargo_target \
    cargo build --release --locked --target x86_64-unknown-linux-musl
COPY server/src ./src
COPY server/templates ./templates
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo_registry \
    --mount=type=cache,target=/app/target,id=cargo_target \
    cargo build --release --locked --target x86_64-unknown-linux-musl && \
    cp target/x86_64-unknown-linux-musl/release/oplint-live /usr/local/bin/

    # ── Stage 3: Production runtime (glibc‑based) ────────────────
    FROM debian:bookworm-slim AS prod
    # Install only the runtime necessities:
    #   ca-certificates - for HTTPS
    #   git - required by oplint
    #   (libssl3 is not needed because oplint-live is musl-static, but it's harmless)
    RUN apt-get update && \
        apt-get install -y --no-install-recommends ca-certificates git && \
        rm -rf /var/lib/apt/lists/* && \
        addgroup --system appgroup && \
        adduser  --system appuser --ingroup appgroup
    # Copy both binaries
    COPY --from=cli-installer /usr/local/bin/oplint      /usr/local/bin/
    COPY --from=builder       /usr/local/bin/oplint-live /usr/local/bin/
    RUN chmod +x /usr/local/bin/oplint /usr/local/bin/oplint-live
    USER appuser
    EXPOSE 8080
    CMD ["oplint-live"]

# ── Stage 4: Development (hot-reload) ────────────────────────
FROM rust:1.93-slim AS dev
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev git && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-watch
COPY --from=cli-installer /usr/local/bin/oplint /usr/local/bin/
WORKDIR /app
CMD ["cargo", "watch", "-x", "run"]
