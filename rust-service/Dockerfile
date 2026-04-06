FROM rust:1.78-alpine AS builder

WORKDIR /app

RUN apk add --no-cache build-base

COPY Cargo.toml ./
COPY Cargo.lock ./
RUN mkdir src \
    && printf 'fn main() {}\n' > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
RUN cargo build --release

FROM alpine:3.22

RUN apk add --no-cache ca-certificates wget

WORKDIR /app

COPY --from=builder /app/target/release/rust-service /usr/local/bin/rust-service

EXPOSE 9000

CMD ["rust-service"]
