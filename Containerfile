FROM rust:1.74.0-alpine3.18 AS prepare

WORKDIR /app

RUN apk add --no-cache musl-dev openssl-dev

FROM prepare AS chef

RUN cargo install cargo-chef

FROM chef AS planner

COPY . .

RUN cargo chef prepare

FROM planner AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release

COPY . .

RUN cargo build --release

FROM alpine:3.18

WORKDIR /app
COPY --from=builder /app/target/release/lure .

ENTRYPOINT [ "/app/lure" ]
