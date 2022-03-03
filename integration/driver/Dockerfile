FROM rust:1.57

WORKDIR /root

COPY . .
RUN cargo build && \
    cp ./target/debug/driver /usr/local/bin/drivern

ENV AWS_DEFAULT_REGION=us-west-1
ENV AWS_SECRET_ACCESS_KEY=dummy
ENV AWS_ACCESS_KEY_ID=dummy

ENTRYPOINT ["driver"]
