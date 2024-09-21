FROM rust:1.80-slim AS builder

RUN apt-get update \
    && apt-get install -y \
        pkg-config \
        libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY . .

RUN cargo install --path .

########################################################################################################################

FROM debian:12-slim

LABEL org.opencontainers.image.source=https://github.com/s373r/freshrss-image-cache-service-rs

RUN apt-get update \
    && apt-get install -y \
        pkg-config \
        libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/freshrss-image-cache-service-rs /usr/local/bin/freshrss-image-cache-service-rs

ENTRYPOINT ["freshrss-image-cache-service-rs"]
