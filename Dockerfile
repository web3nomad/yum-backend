FROM rust:1.74 as builder
WORKDIR /app
COPY . .
RUN cargo build -p api-server --release --target=x86_64-unknown-linux-gnu

FROM ubuntu:latest
RUN apt-get update && apt-get install -y libssl-dev pkg-config
RUN apt-get install -y ca-certificates

WORKDIR /app
COPY --from=builder /app/target/release /app/release
CMD ["./release/api-server"]
# CMD ["cargo", "run", "-p", "api-server", "--release"]
