FROM ubuntu:20.04

WORKDIR /src/
COPY . .

RUN adduser --disabled-password --gecos '' user
USER user

RUN ./setup
