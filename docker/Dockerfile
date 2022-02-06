FROM rust:1.57 AS dev

RUN apt-get update && \
    apt-get install --no-install-recommends -y jq=1.6-2.1 && \
    rm -rf /var/lib/apt/lists/*
RUN rustup component add rustfmt clippy && cargo install --force cargo-make
WORKDIR /usr/src/sqsproxyd

FROM dev AS release-builder

ARG RELEASE_TARGET=x86_64-unknown-linux-gnu
COPY . .
RUN rustup target add ${RELEASE_TARGET} && \
    cargo make --env RELEASE_TARGET=${RELEASE_TARGET} build-release

# hadolint ignore=DL3006
FROM gcr.io/distroless/cc AS sqsproxyd

ARG RELEASE_TARGET=x86_64-unknown-linux-gnu
COPY --from=release-builder /usr/src/sqsproxyd/target/${RELEASE_TARGET}/release/sqsproxyd /bin/sqsproxyd
ENTRYPOINT ["sqsproxyd"]
