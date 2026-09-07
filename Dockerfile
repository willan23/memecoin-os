FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p memecoin-os-api

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /src/target/release/memecoin-os-api /app/memecoin-os-api
COPY tokens /app/tokens
COPY migrations /app/migrations
ENV TOKEN_REGISTRY_PATH=/app/tokens
ENV MIGRATIONS_PATH=/app/migrations
ENV API_HOST=0.0.0.0
ENV RUST_LOG=info
EXPOSE 8080
CMD ["/app/memecoin-os-api"]
