.PHONY: build \
	ci-00-docker

build:
	docker build -f Dockerfile.python .

ci-00-docker:
	test/00-docker
