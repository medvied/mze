.PHONY: build \
	ci-00-license-headers \
	ci-01-docker \
	ci-02-flake8 \
	ci-03-mypy \
	ci-04-build \
	ci-05-upload-local

define run-docker =
	docker run --rm --mount type=bind,src=$(shell pwd),dst=/data
endef

build:
	docker build -f Dockerfile.python .

ci-00-license-headers:
	test/00-license-headers

ci-01-docker:
	test/01-docker

ci-02-flake8:
	$(run-docker) --entrypoint test/02-flake8 mze-check:test

ci-03-mypy:
	$(run-docker) --entrypoint test/03-mypy mze-check:test

ci-04-build:
	$(run-docker) --entrypoint test/04-build mze-build:test

ci-05-upload-local:
	$(run-docker) \
		--env-file cicd/env.local.list \
		--entrypoint test/05-upload-local mze-build:test
