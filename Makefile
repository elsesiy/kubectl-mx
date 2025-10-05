.PHONY: help build test clean release lint fmt fmt-check check run doc install setup-clusters teardown-clusters test-integration

.DEFAULT_GOAL := help

build: ## Build the project in debug mode
	cargo build

test: ## Run all tests
	cargo test

clean: ## Clean build artifacts
	cargo clean

release: ## Build the project in release mode
	cargo build --release

lint: ## Run the linter (clippy)
	cargo clippy -- -D warnings

fmt: ## Format the code
	cargo fmt

fmt-check: ## Check code formatting without modifying files
	cargo fmt --check

check: ## Check code without building
	cargo check

run: ## Run the binary with help
	cargo run -- --help

doc: ## Generate and open documentation
	cargo doc --open

install: ## Install the binary to your system
	cargo install --path .

setup-clusters: ## Setup kind clusters for testing
	./hack/setup-kind-clusters.sh

teardown-clusters: ## Teardown kind clusters
	./hack/teardown-kind-clusters.sh

test-integration: ## Run integration tests (requires clusters to be set up)
	cargo test --test integration

help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
