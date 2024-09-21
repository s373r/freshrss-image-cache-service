FROM rust:1.80-slim AS builder

RUN apt update \
    && apt install -y \
        pkg-config \
        libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY . .

RUN cargo install --path .

########################################################################################################################

FROM debian:12-slim

LABEL org.opencontainers.image.source=https://github.com/s373r/freshrss-image-cache-service-rs

RUN apt update \
    && apt install -y \
        ca-certificates \
        openssl \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

WORKDIR /usr/src/app
COPY --from=builder /usr/local/cargo/bin/freshrss-image-cache-service-rs /usr/local/bin/freshrss-image-cache-service-rs

ENTRYPOINT ["freshrss-image-cache-service-rs"]
