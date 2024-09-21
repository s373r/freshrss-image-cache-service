IMAGE_REPO = ghcr.io/s373r
IMAGE = freshrss-image-cache-service-rs
VERSION = $(shell cargo pkgid | sed 's/.*#//')

.PHONY: lint
lint:
	cargo fmt --check
	cargo clippy --workspace --all-targets -- -D warnings

.PHONY: start
start:
	APP_PORT=3000 \
	APP_ACCESS_TOKEN=TEST_TOKEN \
	APP_IMAGES_DIR=./images \
		cargo run

image image-push: IMAGE_EXISTS = $(shell \
		docker manifest inspect $(IMAGE_REPO)/$(IMAGE):$(VERSION) > /dev/null 2>&1 \
			&& echo yes \
	)

.PHONY: image
image:
	@if [ "$(IMAGE_EXISTS)" = "yes" ]; then \
		echo "The image with version $(VERSION) is already existed"; \
		echo "Looks like you forgot to update Cargo.toml"; \
		exit 1; \
	fi

	docker build -t $(IMAGE_REPO)/$(IMAGE):$(VERSION) .

.PHONY: image-push
image-push:
	@if [ "$(IMAGE_EXISTS)" = "yes" ]; then \
		echo "The image with version $(VERSION) is already existed"; \
		echo "Looks like you forgot to update Cargo.toml"; \
		exit 1; \
	fi

	docker push $(IMAGE_REPO)/$(IMAGE):$(VERSION)

	docker tag $(IMAGE_REPO)/$(IMAGE):$(VERSION) $(IMAGE_REPO)/$(IMAGE):latest
	docker push $(IMAGE_REPO)/$(IMAGE):latest
