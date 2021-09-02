.PHONY: build \
	test \
	ci-00-license-headers \
	ci-01-docker \
	ci-02-flake8 \
	ci-03-mypy \
	ci-04-build \
	ci-05-upload-local \
	ci-10-install \
	ci-20-unit \
	ci-30-integration

define run-docker =
	docker run --rm --mount type=bind,src=$(shell pwd),dst=/data
endef

build:
	docker build -f Dockerfile.python .

test: ci-00-license-headers \
	ci-02-flake8 ci-03-mypy \
	ci-04-build ci-05-upload-local ci-10-install \
	ci-20-unit ci-30-integration ;

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

ci-10-install:
	$(run-docker) \
		--env-file cicd/env.local.list \
		--entrypoint test/10-install mze:test

ci-20-unit:
	$(run-docker) --entrypoint test/20-unit mze:test

ci-30-integration:
	$(run-docker) --entrypoint test/30-integration mze:test
