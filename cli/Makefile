.PHONY: help install install-prod lint lint-fix format format-check type-check check test test-cov clean

DIRS ?= src tests

help: ## Show this help message
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

install: ## Install dependencies
	uv sync

install-prod: ## Install production dependencies
	uv sync --no-dev

lint: ## Run linting with ruff
	uv run scripts/lint_ruff.sh

lint-fix: ## Run linting with ruff and fix auto-fixable issues
	uv run scripts/lint_ruff.sh --fix

format: ## Format code with ruff
	uv run scripts/format_ruff.sh

format-check: ## Check formatting with ruff
	uv run scripts/format_ruff.sh --check

type-check: ## Run type checking with mypy
	uv run scripts/mypy.sh

check: ## Run all checks (lint + format check + type check + test)
	@$(MAKE) lint
	@$(MAKE) format-check
	@$(MAKE) type-check
	@$(MAKE) test

test: ## Run tests
	uv run scripts/pytest.sh

test-cov: ## Run tests with coverage
	uv run scripts/pytest.sh -c

clean: ## Clean up cache and temporary files
	find . -type d -name "__pycache__" -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete
	find . -type d -name "*.egg-info" -exec rm -rf {} +
	rm -rf .pytest_cache/
	rm -rf .ruff_cache/
	rm -rf .mypy_cache/
	rm -rf htmlcov/ coverage/
	rm -rf .coverage coverage.xml
