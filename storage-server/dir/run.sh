#!/usr/bin/env bash
set -eux

python -m flake8 *.py
python -m mypy --strict --warn-unreachable \
	--show-error-context --show-error-codes --pretty *.py

exec python main.py
