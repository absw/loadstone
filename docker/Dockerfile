FROM ubuntu:20.04

WORKDIR /src/
COPY . .

RUN ./rootsetup

USER user
RUN ./setup
