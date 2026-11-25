SHELL := /bin/bash

ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
PYTHON ?= python3

.DEFAULT_GOAL := run

.PHONY: help run check clean

help:
	@printf '%s\n' \
		'Usage:' \
		'  make [target]' \
		'' \
		'Targets:' \
		'  help     Show this help' \
		'  run      Launch degima-cursor (default; prompts for SSH command)' \
		'  check    Syntax-check degima_cursor/ with compileall' \
		'  clean    Remove __pycache__, .local, and target' \
		'' \
		'Examples:' \
		'  make run' \
		'  $(PYTHON) -m degima_cursor "ssh root@host -p 10008"' \
		'  $(PYTHON) -m degima_cursor --ssh-key ~/.ssh/id_ed25519 "ssh root@host -p 10008"' \
		'' \
		'CLI:'
	@$(PYTHON) -m degima_cursor --help || true

run:
	@$(PYTHON) -m degima_cursor

check:
	@$(PYTHON) -m compileall -q "$(ROOT)/degima_cursor"

clean:
	@find "$(ROOT)/degima_cursor" -type d -name '__pycache__' -exec rm -rf {} + 2>/dev/null || true
	@rm -rf "$(ROOT)/.local" "$(ROOT)/target"
