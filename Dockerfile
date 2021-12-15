FROM rust:1.57 AS dev

WORKDIR /usr/src/sqsproxyd
COPY Cargo.toml Cargo.lock ./
