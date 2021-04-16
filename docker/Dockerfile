FROM ubuntu:20.04

WORKDIR /src/
COPY . .

RUN adduser --disabled-password --gecos '' user
RUN usermod -aG sudo user
USER user

RUN ./setup
