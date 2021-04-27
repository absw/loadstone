FROM rust:buster

# Add user
RUN adduser --disabled-password --gecos '' user
RUN usermod -aG sudo user
USER user
WORKDIR /home/user/

# Enable SSH Agent
RUN eval `ssh-agent`

# Install rust dependencies
RUN rustup default nightly
RUN rustup update
RUN cargo install cargo-binutils
RUN rustup component add llvm-tools-preview
RUN rustup target add thumbv7em-none-eabi
