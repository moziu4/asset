# Dockerfile for Backend
FROM rust:slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential\
    cmake \
    nasm

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY . .
RUN cargo build --release

# Etapa 2: Imagen final (producción)
FROM rust:slim

RUN apt-get update && apt-get install -y \
    openssl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/asset .

CMD ["./asset"]