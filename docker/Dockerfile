FROM rust:1.57 AS dev

RUN rustup component add rustfmt clippy
RUN cargo install --force cargo-make
WORKDIR /usr/src/sqsproxyd

FROM dev AS builder

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build && \
    rm -rf target/release/deps/sqsproxyd*
COPY . .
RUN cargo build

FROM dev AS release-builder

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf target/release/deps/sqsproxyd*
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc AS sqsproxyd

COPY --from=release-builder /usr/src/sqsproxyd/target/release/sqsproxyd /bin/sqsproxyd
CMD ["/bin/sqsproxyd"]
