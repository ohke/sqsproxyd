FROM rust:1.57

WORKDIR /root

COPY . .
RUN cargo build && \
    cp ./target/debug/api /usr/local/bin/api

ENTRYPOINT ["api"]
