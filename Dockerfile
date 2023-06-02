# fetch the vendor with the builder platform to avoid qemu issues
FROM --platform=$BUILDPLATFORM rust:1-alpine AS vendor

WORKDIR /code
RUN cargo init
COPY Cargo.lock /code/Cargo.lock
COPY Cargo.toml /code/Cargo.toml

RUN cargo init --lib donos-blocklist-loader
COPY donos-blocklist-loader/Cargo.toml /code/donos-blocklist-loader/Cargo.toml

RUN cargo init --lib donos-parser
COPY donos-parser/Cargo.toml /code/donos-parser/Cargo.toml

# https://docs.docker.com/engine/reference/builder/#run---mounttypecache
RUN --mount=type=cache,target=$CARGO_HOME/git,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/registry,sharing=locked \
    mkdir -p /code/.cargo \
    && cargo vendor > /code/.cargo/config


FROM rust:1-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /code

COPY donos-blocklist-loader/src /code/donos-blocklist-loader/src
COPY donos-blocklist-loader/Cargo.toml /code/donos-blocklist-loader/Cargo.toml

COPY donos-parser/benches /code/donos-parser/benches
COPY donos-parser/fuzz /code/donos-parser/fuzz
COPY donos-parser/src /code/donos-parser/src
COPY donos-parser/Cargo.toml /code/donos-parser/Cargo.toml

COPY migrations /code/migrations
COPY src /code/src
COPY Cargo.lock Cargo.toml /code/

COPY --from=vendor /code/.cargo /code/.cargo
COPY --from=vendor /code/vendor /code/vendor

RUN --mount=type=cache,target=/code/target/release/.fingerprint,sharing=private \
    --mount=type=cache,target=/code/target/release/build,sharing=private \
    --mount=type=cache,target=/code/target/release/deps,sharing=private \
    --mount=type=cache,target=/code/target/release/examples,sharing=private \
    --mount=type=cache,target=/code/target/release/incremental,sharing=private \
    cargo build --release --offline

FROM alpine

ENV CONFIG_PATH=/etc/donos/donos.toml
ENV DATABASE_URL=/etc/donos/database.db
ENV DNS_HOST=0.0.0.0
ENV DNS_PORT=53

COPY donos.toml /etc/donos/donos.toml
COPY --from=builder /code/target/release/donos /usr/bin/donos

EXPOSE 53/udp

ENTRYPOINT [ "/usr/bin/donos" ]
CMD [ "dns" ]
