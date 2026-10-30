# Jukelight: one image with the backend and the embedded kiosk.
# Build with `docker compose up --build` (see README).

# 1. Kiosk (React) -----------------------------------------------------------
FROM node:24-slim AS kiosk
WORKDIR /src/web/kiosk
COPY web/kiosk/package.json web/kiosk/package-lock.json ./
RUN npm ci
COPY web/kiosk/ ./
RUN npm run build

# 2. Rust dependencies, cached separately from Jukelight's own code ----------
FROM rust:1-slim-trixie AS chef
RUN cargo install cargo-chef --locked
WORKDIR /src

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
# aws-lc-rs (TLS for artwork downloads) compiles C code.
RUN apt-get update \
    && apt-get install -y --no-install-recommends cmake clang \
    && rm -rf /var/lib/apt/lists/*
COPY --from=planner /src/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY --from=kiosk /src/web/kiosk/dist web/kiosk/dist
RUN cargo build --release --bin jukelight

# 3. Runtime -----------------------------------------------------------------
FROM debian:trixie-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --no-create-home --shell /usr/sbin/nologin jukelight
COPY --from=builder /src/target/release/jukelight /usr/local/bin/jukelight
USER jukelight
ENV HTTP_PORT=8080
EXPOSE 8080
# No curl in the image: a plain HTTP request through bash.
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["bash", "-c", "exec 3<>/dev/tcp/127.0.0.1/${HTTP_PORT} && printf 'GET /api/v1/player HTTP/1.0\\r\\n\\r\\n' >&3 && head -n 1 <&3 | grep -q ' 200 '"]
ENTRYPOINT ["/usr/local/bin/jukelight"]
