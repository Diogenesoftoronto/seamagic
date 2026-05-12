FROM rust:1.85-slim-bookworm as builder

WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release
RUN cp target/release/seamagic /bin/seamagic

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /bin/seamagic /bin/seamagic
EXPOSE 3000
CMD ["/bin/seamagic", "http-mcp", "--bind", "0.0.0.0:3000"]
