FROM rust:1.62 as build

RUN USER=root cargo new --bin mcstarter
WORKDIR /mcstarter

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release && rm ./src/*.rs ./target/release/deps/mcstarter*

COPY . ./
RUN cargo build --release


FROM debian:11-slim
RUN apt-get update && \
    apt-get install --no-install-recommends -y ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

COPY --from=build /mcstarter/target/release/mcstarter /usr/local/bin

CMD [ "/usr/local/bin/mcstarter" ]
