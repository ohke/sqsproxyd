FROM rust:1.57 AS builder

WORKDIR /usr/src/sqsproxyd
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf target/release/deps/sqsproxyd*
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc

COPY --from=builder /usr/src/sqsproxyd/target/release/sqsproxyd /bin/sqsproxyd
CMD ["/bin/sqsproxyd"]
