FROM rust:1.62 as build

# create a new empty shell project
RUN USER=root cargo new --bin mcstarter
WORKDIR /mcstarter

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release && rm ./src/*.rs ./target/release/deps/mcstarter*

# copy your source tree
COPY . ./

# build for release
RUN cargo build --release

# our final image
FROM debian:11-slim

# copy the build artifact from the build stage
COPY --from=build /mcstarter/target/release/mcstarter /usr/local/bin

# set the startup command to run your binary
CMD [ "/usr/local/bin/mcstarter" ]
