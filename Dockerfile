# Stage 1: Build
FROM rust:1.88 AS builder
WORKDIR /build

# Copy workspace Cargo files first (for dependency caching)
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY src/ src/
COPY data/ data/   # ✅ quan trọng: include_str! cần file tồn tại khi compile

# Build release binaries
RUN cargo build --release --bin bizclaw --bin bizclaw-platform

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/bizclaw /usr/local/bin/bizclaw
COPY --from=builder /build/target/release/bizclaw-platform /usr/local/bin/bizclaw-platform

RUN mkdir -p /root/.bizclaw

ENV BIZCLAW_CONFIG=/root/.bizclaw/config.toml
ENV RUST_LOG=info
ENV TZ=Asia/Ho_Chi_Minh

EXPOSE 3001 10001 10002 10003 10004 10005

# ❌ Bỏ HEALTHCHECK trong Dockerfile để tránh hardcode port.
# Railway sẽ dùng healthcheckPath=/health từ railway.toml.

ENTRYPOINT ["/usr/local/bin/bizclaw-platform"]
CMD ["--port", "3001", "--bizclaw-bin", "/usr/local/bin/bizclaw"]
