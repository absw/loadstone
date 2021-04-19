FROM ubuntu:20.04

WORKDIR /src/
COPY . .

RUN ./rootsetup

USER user
RUN ./setup

# Add cargo bin directory to PATH.
ENV PATH="/home/user/.cargo/bin:${PATH}"
