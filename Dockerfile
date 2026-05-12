FROM rust:1.88-slim-bookworm as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo build --release && cp target/release/seamagic /seamagic

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /seamagic /seamagic
EXPOSE 3000
CMD ["/seamagic", "http-mcp", "--bind", "0.0.0.0:3000"]
