# Multi-stage build for the axum API (workspace).
FROM rust:1.96-slim AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p clinical-trial-matcher

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/clinical-trial-matcher /usr/local/bin/
WORKDIR /app
EXPOSE 8002
CMD ["clinical-trial-matcher"]
