.PHONY: build \
	ci-00-docker \
	ci-01-flake8 \
	ci-02-mypy \
	ci-03-build

define run-docker =
	docker run --rm --mount type=bind,src=$(shell pwd),dst=/data
endef

build:
	docker build -f Dockerfile.python .

ci-00-docker:
	test/00-docker

ci-01-flake8:
	$(run-docker) --entrypoint test/01-flake8 mze-check:test

ci-02-mypy:
	$(run-docker) --entrypoint test/02-mypy mze-check:test

ci-03-build:
	$(run-docker) --entrypoint test/03-build mze-build:test
