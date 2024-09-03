FROM rust:1 AS chef

WORKDIR /usr/src/lure

RUN cargo install cargo-chef --locked

FROM chef AS prep

COPY . .

RUN cargo chef prepare

FROM chef AS cook

COPY --from=prep /usr/src/lure/recipe.json recipe.json

RUN cargo chef cook --release

COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12

ENV RUST_LOG="lure=info"

WORKDIR /app
COPY --from=cook /usr/src/lure/target/release/lure /app

VOLUME [ "/app" ]

CMD ["./lure", "start"]
