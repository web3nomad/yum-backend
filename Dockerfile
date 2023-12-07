FROM rust:1.74.0-bookworm AS builder

WORKDIR /app
COPY . .
RUN cargo build -p api-server --locked --release


FROM debian:bookworm-slim AS final

RUN apt-get update && apt-get install -y libssl-dev pkg-config
RUN apt-get install -y ca-certificates

RUN apt-get install -y libmariadb-dev-compat libmariadb-dev

WORKDIR /app
COPY --from=builder /app/target/release/api-server /app
CMD ["./api-server"]
