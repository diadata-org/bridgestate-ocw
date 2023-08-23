# This is the build stage for Substrate. Here we create the binary.
FROM docker.io/library/ubuntu:20.04 as builder


RUN apt-get update -y && \
	apt-get install -y automake build-essential apt-utils curl  libssl1.1 ca-certificates -y \
	clang git make libssl-dev llvm libudev-dev protobuf-compiler && \
	apt-get clean


RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup toolchain install nightly-2023-03-20 --force
RUN rustup default nightly-2023-03-20
RUN rustup component add rust-src
RUN rustup update nightly && \
	rustup update stable && \
	rustup target add wasm32-unknown-unknown --toolchain nightly-2023-03-20

WORKDIR /substrate
COPY . /substrate

RUN cargo build  --release

RUN  cp /substrate/target/release/node-template /usr/local/bin


EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]