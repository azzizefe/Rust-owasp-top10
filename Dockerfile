# Build stage
FROM rust:1-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd -m -u 10001 appuser
COPY --from=builder /app/target/release/owasp-web /usr/local/bin/app
COPY --from=builder /app/migrations /migrations
COPY --from=builder /app/crates/web/templates /templates
USER appuser
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/app"]
