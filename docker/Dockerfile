FROM rust:buster

RUN rustup default nightly
RUN rustup update
RUN cargo install cargo-binutils
RUN rustup component add llvm-tools-preview
RUN rustup target add thumbv7em-none-eabi
