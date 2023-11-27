FROM rust:1.74

RUN apt-get update
RUN apt-get install -y protobuf-compiler

WORKDIR /app

COPY third_parties /app/third_parties
COPY build.rs /app/build.rs
COPY src /app/src
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.toml /app/Cargo.toml

RUN cargo update
RUN cargo build

CMD cargo run
