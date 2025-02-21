FROM rust:1 AS chef

WORKDIR /usr/src/lure

RUN cargo install cargo-chef --locked

FROM chef AS prep

COPY . .

RUN cargo chef prepare

FROM prep AS cook

COPY --from=prep /usr/src/lure/recipe.json recipe.json

RUN cargo chef cook --release

FROM cook AS server

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12

# ENV LURE_LOG="info"

WORKDIR /app
COPY --from=server /usr/src/lure/target/release/lure /app

VOLUME [ "/app" ]

CMD ["./lure", "start"]
