FROM rust:1.80 AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM alpine:3.14
LABEL org.opencontainers.image.source=https://github.com/s373r/freshrss-image-cache-service-rs
COPY --from=builder /usr/local/cargo/bin/freshrss-image-cache-service-rs /usr/local/bin/freshrss-image-cache-service-rs
ENTRYPOINT ["freshrss-image-cache-service-rs"]
