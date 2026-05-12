# Build stage - use newer Rust
FROM ghcr.io/cargo-rs/cargo:1.88-slim-bookworm AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml ./
COPY Cargo.lock ./
COPY src ./src
RUN cargo build --release && mkdir -p /out && cp target/release/seamagic /out/

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /out/seamagic /seamagic
EXPOSE 3000
CMD ["/seamagic", "http-mcp", "--bind", "0.0.0.0:3000"]
