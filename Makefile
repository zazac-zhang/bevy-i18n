# Traditional Makefile for bevy_i18n
# Use 'just' for modern commands, or 'make' for traditional workflow

.PHONY: help build test clean fmt clippy docs i18n-check ci install-tools

# Default target
help:
	@echo "bevy_i18n Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  make build        - Build project"
	@echo "  make test         - Run tests"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make fmt          - Format code"
	@echo "  make clippy       - Run clippy"
	@echo "  make docs         - Generate documentation"
	@echo "  make i18n-check   - Validate i18n files"
	@echo "  make ci           - Run CI checks"
	@echo "  make install-tools - Install development tools"
	@echo ""
	@echo "For more commands, use 'justfile' instead: just"

# Build targets
build:
	cargo build --all-features

build-release:
	cargo build --release --all-features

# Test targets
test:
	cargo test --all-features

test-verbose:
	cargo test --all-features -- --nocapture

# Code quality
fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --all-features -- -D warnings

# Documentation
docs:
	cargo doc --no-deps --all-features

docs-open:
	cargo doc --no-deps --all-features --open

# i18n targets
i18n-tools:
	cargo build --release --bin i18n-extract --bin i18n-validate

i18n-extract: i18n-tools
	./target/release/i18n-extract src locales/template.yaml

i18n-validate: i18n-tools
	./target/release/i18n-validate assets/locales

i18n-check: i18n-extract i18n-validate

# CI targets
ci: fmt-check clippy test i18n-check

# Development tools
install-tools:
	cargo install cargo-watch cargo-audit cargo-outdated
	@echo "Tools installed successfully"

# Cleanup
clean:
	cargo clean
	rm -rf locales/

# Show project info
info:
	@echo "Project: bevy_i18n"
	@echo "Version: $$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)"
	@echo "Rust: $$(rustc --version)"
	@echo "Cargo: $$(cargo --version)"
