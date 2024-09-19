IMAGE_REPO = ghcr.io/s373r
IMAGE = freshrss-image-cache-service-rs
VERSION = $(shell cargo pkgid | sed 's/.*#//')

.PHONY: lint
lint:
	cargo fmt --check
	cargo clippy --workspace --all-targets -- -D warnings

.PHONY: lint
start:
	cargo run

.PHONY: image
image:
	docker build -t $(IMAGE_REPO)/$(IMAGE):$(VERSION) .

.PHONY: kamu-base-push
image-push:
	docker push $(IMAGE_REPO)/$(IMAGE):$(VERSION)

	docker tag $(IMAGE_REPO)/$(IMAGE):$(VERSION) $(IMAGE_REPO)/$(IMAGE):latest
	docker push $(IMAGE_REPO)/$(IMAGE):latest
