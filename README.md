# freshrss-image-cache-service-rs

`freshrss-image-cache-service-rs` is a simple service for caching images locally, specifically designed for
the [freshrss-image-cache-pluginx](https://github.com/Victrid/freshrss-image-cache-plugin) extension. This can be
particularly useful in the case of time-limited links to images (e.g. in the case of [rsshub.app](https://rsshub.app/))

## Quick Start

To start the service locally for development, run just one command:

```shell
make start
```

## Docker Image

Here is an example of a Docker Compose configuration for quick deployment:

```yaml
# $ cat compose.yaml

services:
  cache_server:
    image: ghcr.io/s373r/freshrss-image-cache-service-rs:latest
    ports:
      - 3000:3000
    volumes:
      - ./images:/usr/src/app/images
    environment:
      - APP_PORT=3000
      - APP_ACCESS_TOKEN=TODO_REPLACE_ME_WITH_RANDOM_VALUE
      - APP_IMAGES_DIR=./images
```

NOTE: Make sure to replace APP_ACCESS_TOKEN with a unique value!

## Release procedure

```shell
make lint
make image
make image-push
```
