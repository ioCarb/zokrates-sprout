# 1. This tells docker to use the Rust official image
#FROM rust:slim-bullseye
FROM rust:bullseye

WORKDIR /zokrates

# 2. Copy the files in your machine to the Docker image
COPY ./ /zokrates

RUN ls -al /zokrates

RUN apt-get update && apt-get upgrade -y
RUN apt-get -y install protobuf-compiler

# Build your program for release
RUN cargo build --release

EXPOSE 4004

# Run the binary
CMD ["/zokrates/target/release/zokrates"]
