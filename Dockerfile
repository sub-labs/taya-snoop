FROM rust:latest AS chef

RUN cargo install cargo-chef 
WORKDIR /app

FROM chef AS planner

COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:stable AS runtime

RUN apt update && apt install -y libpq5 ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/snoop /usr/local/bin/