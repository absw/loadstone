FROM rust:buster

# Install zip
RUN apt-get update
RUN apt-get install zip -y

# Install rust dependencies
RUN rustup default nightly
RUN rustup update
RUN cargo install cargo-binutils
RUN rustup component add llvm-tools-preview
RUN rustup component add rustfmt
RUN rustup target add thumbv7em-none-eabi
RUN rustup target add thumbv7em-none-eabihf
