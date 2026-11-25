SHELL := /bin/bash

ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
PYTHON ?= python3

.DEFAULT_GOAL := run

.PHONY: run check clean

run:
	@$(PYTHON) -m degima_cursor

check:
	@$(PYTHON) -m compileall -q "$(ROOT)/degima_cursor"

clean:
	@find "$(ROOT)/degima_cursor" -type d -name '__pycache__' -exec rm -rf {} + 2>/dev/null || true
	@rm -rf "$(ROOT)/.local" "$(ROOT)/target"
