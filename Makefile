# Omni Universal Package Manager - Makefile

# Configuration
CARGO := cargo
BINARY_NAME := omni
VERSION := $(shell grep '^version' Cargo.toml | head -n1 | cut -d'"' -f2)
BUILD_DIR := target/release
PACKAGE_DIR := packages
SCRIPT_DIR := scripts

# Default features for different builds
MINIMAL_FEATURES := --no-default-features
DEFAULT_FEATURES := 
GUI_FEATURES := --features gui
SSH_FEATURES := --features ssh
ALL_FEATURES := --all-features

# Cross-compilation targets
LINUX_X64 := x86_64-unknown-linux-gnu
LINUX_ARM64 := aarch64-unknown-linux-gnu
MACOS_X64 := x86_64-apple-darwin
MACOS_ARM64 := aarch64-apple-darwin
WINDOWS_X64 := x86_64-pc-windows-msvc

.PHONY: all build test lint audit clean help install package deps format check

# Default target
all: deps test lint build package

# Help target
help:
	@echo "Omni Universal Package Manager - Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  all        - Complete build pipeline (deps, test, lint, build, package)"
	@echo "  build      - Build release binary for current platform"
	@echo "  build-all  - Build for all supported platforms"
	@echo "  test       - Run all tests"
	@echo "  lint       - Run code linting (rustfmt + clippy)"
	@echo "  audit      - Run security audit"
	@echo "  check      - Quick compile check"
	@echo "  format     - Format code with rustfmt"
	@echo "  clean      - Clean build artifacts"
	@echo "  install    - Install binary to local system"
	@echo "  package    - Create distribution packages"
	@echo "  deps       - Install build dependencies"
	@echo ""
	@echo "Build variants:"
	@echo "  build-minimal  - Build without default features"
	@echo "  build-gui      - Build with GUI support"
	@echo "  build-ssh      - Build with SSH support"
	@echo "  build-full     - Build with all features"
	@echo ""
	@echo "Platform-specific builds:"
	@echo "  build-linux    - Build for Linux (x64 + ARM64)"
	@echo "  build-macos    - Build for macOS (x64 + ARM64)"
	@echo "  build-windows  - Build for Windows (x64)"
	@echo ""
	@echo "Environment variables:"
	@echo "  CARGO_FEATURES - Additional cargo features"
	@echo "  BUILD_MODE     - 'debug' or 'release' (default: release)"

# Dependencies
deps:
	@echo "Installing build dependencies..."
	rustup target add $(LINUX_X64) || true
	rustup target add $(LINUX_ARM64) || true
	rustup target add $(MACOS_X64) || true
	rustup target add $(MACOS_ARM64) || true
	rustup target add $(WINDOWS_X64) || true
	rustup component add rustfmt clippy
	command -v cargo-audit >/dev/null 2>&1 || cargo install cargo-audit
	command -v cross >/dev/null 2>&1 || cargo install cross

# Quick compile check
check:
	@echo "Running compile check..."
	$(CARGO) check $(MINIMAL_FEATURES)

# Format code
format:
	@echo "Formatting code..."
	$(CARGO) fmt --all

# Linting
lint: format
	@echo "Running linting..."
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy --all-targets $(MINIMAL_FEATURES) -- -D warnings

# Security audit
audit:
	@echo "Running security audit..."
	$(CARGO) audit

# Tests
test:
	@echo "Running tests..."
	$(CARGO) test $(MINIMAL_FEATURES) --verbose
	$(CARGO) test --doc $(MINIMAL_FEATURES)

# Build variants
build: build-minimal

build-minimal:
	@echo "Building minimal version..."
	$(CARGO) build --release $(MINIMAL_FEATURES)

build-gui:
	@echo "Building with GUI support..."
	$(CARGO) build --release $(GUI_FEATURES)

build-ssh:
	@echo "Building with SSH support..."
	$(CARGO) build --release $(SSH_FEATURES)

build-full:
	@echo "Building with all features..."
	$(CARGO) build --release $(ALL_FEATURES)

# Platform-specific builds
build-linux:
	@echo "Building for Linux platforms..."
	$(CARGO) build --release --target $(LINUX_X64) $(MINIMAL_FEATURES)
	cross build --release --target $(LINUX_ARM64) $(MINIMAL_FEATURES) || echo "ARM64 build failed"

build-macos:
	@echo "Building for macOS platforms..."
	$(CARGO) build --release --target $(MACOS_X64) $(MINIMAL_FEATURES)
	$(CARGO) build --release --target $(MACOS_ARM64) $(MINIMAL_FEATURES) || echo "ARM64 build failed"

build-windows:
	@echo "Building for Windows..."
	$(CARGO) build --release --target $(WINDOWS_X64) $(MINIMAL_FEATURES)

build-all: build-linux build-macos build-windows

# Installation
install: build-minimal
	@echo "Installing omni..."
	sudo cp $(BUILD_DIR)/$(BINARY_NAME) /usr/local/bin/
	sudo chmod +x /usr/local/bin/$(BINARY_NAME)
	@echo "Omni installed to /usr/local/bin/$(BINARY_NAME)"

install-user: build-minimal
	@echo "Installing omni for current user..."
	mkdir -p $$HOME/.local/bin
	cp $(BUILD_DIR)/$(BINARY_NAME) $$HOME/.local/bin/
	chmod +x $$HOME/.local/bin/$(BINARY_NAME)
	@echo "Omni installed to $$HOME/.local/bin/$(BINARY_NAME)"
	@echo "Make sure $$HOME/.local/bin is in your PATH"

# Packaging
package: build-all
	@echo "Creating distribution packages..."
	mkdir -p $(PACKAGE_DIR)
	$(SCRIPT_DIR)/build.sh package

# Development helpers
dev-build:
	$(CARGO) build $(MINIMAL_FEATURES)

dev-run: dev-build
	$(BUILD_DIR)/../debug/$(BINARY_NAME) --help

dev-install: dev-build
	cp target/debug/$(BINARY_NAME) $$HOME/.local/bin/$(BINARY_NAME)-dev
	chmod +x $$HOME/.local/bin/$(BINARY_NAME)-dev

# Benchmarking
bench:
	@echo "Running benchmarks..."
	$(CARGO) bench $(MINIMAL_FEATURES)

# Documentation
docs:
	@echo "Building documentation..."
	$(CARGO) doc --no-deps $(MINIMAL_FEATURES)

docs-open: docs
	$(CARGO) doc --no-deps $(MINIMAL_FEATURES) --open

# Release preparation
release-prep: lint audit test
	@echo "Preparing release $(VERSION)..."
	@echo "Version: $(VERSION)"
	@echo "All checks passed. Ready for release."

# Cleaning
clean:
	@echo "Cleaning build artifacts..."
	$(CARGO) clean
	rm -rf $(PACKAGE_DIR)

clean-all: clean
	@echo "Deep cleaning..."
	rm -rf target/
	rm -rf Cargo.lock

# CI/CD helpers
ci-test: deps test lint audit

ci-build: deps build-all

ci-package: ci-build package

# Docker helpers (if needed)
docker-build:
	docker build -t omni:latest .

docker-run:
	docker run --rm -it omni:latest

# Show build information
info:
	@echo "Build Information:"
	@echo "  Version: $(VERSION)"
	@echo "  Cargo: $$(cargo --version)"
	@echo "  Rustc: $$(rustc --version)"
	@echo "  Build Dir: $(BUILD_DIR)"
	@echo "  Package Dir: $(PACKAGE_DIR)"
	@echo "  Features: $(CARGO_FEATURES)"

# Validate Cargo.toml
validate:
	@echo "Validating Cargo.toml..."
	$(CARGO) verify-project

# Update dependencies
update:
	@echo "Updating dependencies..."
	$(CARGO) update

# Show dependency tree
deps-tree:
	$(CARGO) tree

# Show outdated dependencies
deps-outdated:
	command -v cargo-outdated >/dev/null 2>&1 || cargo install cargo-outdated
	cargo outdated