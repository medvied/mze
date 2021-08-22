.PHONY: build ci-docker-images-nocache ci-docker-images

build:
	docker build -f Dockerfile.python .

ci-docker-images-nocache:
	docker build --file Dockerfile.python --no-cache --pull --target check_and_build --tag mze-check:test .

ci-docker-images:
	docker build --file Dockerfile.python --target check_and_build --tag mze-check:test .
