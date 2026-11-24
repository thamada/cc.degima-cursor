SHELL := /bin/bash

ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
LOCAL_DIR := $(ROOT)/.local
CARGO_HOME := $(LOCAL_DIR)/cargo
RUSTUP_HOME := $(LOCAL_DIR)/rustup
CARGO := $(CARGO_HOME)/bin/cargo

export CARGO_HOME
export RUSTUP_HOME
export PATH := $(CARGO_HOME)/bin:$(PATH)

.DEFAULT_GOAL := run

.PHONY: install run build release clean check

install:
	@./install.sh

$(CARGO):
	@./install.sh

run: build
	@$(CARGO) run --quiet --manifest-path "$(ROOT)/Cargo.toml"

build: $(CARGO)
	@$(CARGO) build --manifest-path "$(ROOT)/Cargo.toml"

release: $(CARGO)
	@$(CARGO) build --release --manifest-path "$(ROOT)/Cargo.toml"

check: $(CARGO)
	@$(CARGO) check --manifest-path "$(ROOT)/Cargo.toml"

clean:
	@if [[ -x "$(CARGO)" ]]; then \
		$(CARGO) clean --manifest-path "$(ROOT)/Cargo.toml"; \
	fi
	@rm -rf "$(LOCAL_DIR)"
