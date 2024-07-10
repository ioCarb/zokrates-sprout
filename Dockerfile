#FROM rust:bullseye
FROM rust:slim-bullseye

WORKDIR /zokrates

COPY ./ /zokrates

RUN apt-get update && apt-get upgrade -y
RUN apt-get -y install protobuf-compiler

RUN cargo build --release

EXPOSE 4001

ENV RUST_LOG=info

ENTRYPOINT ["/zokrates/target/release/zokrates"]
