# Default target when you just run 'make'
.DEFAULT_GOAL := build

# Build the project
build:
	wasm-pack build --target web

# Clean build artifacts
clean:
	rm -rf pkg
	rm -rf target

# Install wasm-pack if not present
install-wasm-pack:
	cargo install wasm-pack

export-url:
	$env:API_URL = "http://localhost:7000/upload"

# Phony targets (not actual files)
.PHONY: build clean install-wasm-pack