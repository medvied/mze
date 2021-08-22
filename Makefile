.PHONY: build ci-docker-images

build:
	docker build -f Dockerfile.python .

ci-docker-images:
	docker build --file Dockerfile.python --no-cache --pull --target check_and_build --tag mze-check:test .
