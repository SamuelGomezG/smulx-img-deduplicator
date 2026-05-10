BINARY_NAME := smulx-dedup
INSTALL_DIR ?= $(HOME)/.local/bin
GALLERY_PATH ?= ~/Pictures

.PHONY: all check fmt lint test build install dev clean

all: build

check: fmt lint test

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

test:
	cargo test --all

build:
	cargo build --release

install: build
	install -d $(INSTALL_DIR)
	install -m 755 target/release/$(BINARY_NAME) $(INSTALL_DIR)/$(BINARY_NAME)

dev:
	cargo run --release -- $(GALLERY_PATH)

clean:
	cargo clean
