FROM rust:buster

# Install zip
RUN apt-get update
RUN apt-get install zip -y

# Add user
RUN adduser --disabled-password --gecos '' user
USER user
WORKDIR /home/user/

RUN eval `ssh-agent -s`

# Install rust dependencies
RUN rustup default nightly
RUN rustup update
RUN cargo install cargo-binutils
RUN rustup component add llvm-tools-preview
RUN rustup target add thumbv7em-none-eabi
RUN rustup target add thumbv7em-none-eabihf
