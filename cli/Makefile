.PHONY: help build release check test clippy fmt fmt-check clean

help: ## Show this help message
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build debug binary
	mise exec -- cargo build

release: ## Build release binary
	mise exec -- cargo build --release

test: ## Run test suite
	mise exec -- cargo test

clippy: ## Run clippy linter
	mise exec -- cargo clippy --all-targets -- -D warnings

fmt: ## Format code with rustfmt
	mise exec -- cargo fmt

fmt-check: ## Check code formatting
	mise exec -- cargo fmt --check

check: ## Run all checks (clippy + fmt-check + test)
	@$(MAKE) fmt-check
	@$(MAKE) clippy
	@$(MAKE) test

clean: ## Clean build artifacts
	mise exec -- cargo clean
