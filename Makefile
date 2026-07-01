# TBox developer tasks.
#
# Conventions:
#   - `make help`           list available targets
#   - `make dev`            start Tauri in development mode
#   - `make build`          build a release Tauri bundle
#   - `make check`          fast Rust type-check + Vue type-check
#   - `make lint`           clippy + cargo fmt --check
#   - `make test`           run the Rust test suite
#   - `make clean`          remove build artifacts (target/, dist/, node_modules/)

SHELL := /bin/zsh
.SHELLFLAGS := -eu -o pipefail -c

# ---- Toolchain --------------------------------------------------------------

PNPM        ?= pnpm
CARGO       ?= cargo
TAURI       ?= $(PNPM) tauri
NODE        ?= node

BACKEND_DIR := src-tauri
FRONTEND    := .

# Colored output (no-op when stdout is not a TTY).
BOLD  := \033[1m
DIM   := \033[2m
RESET := \033[0m

# ---- Default goal -----------------------------------------------------------

.DEFAULT_GOAL := help

.PHONY: help
help: ## list available targets
	@printf "$(BOLD)TBox$(RESET) developer tasks\n\n"
	@printf "$(DIM)Usage:$(RESET) make <target>\n\n"
	@awk 'BEGIN {FS = ":.*?## "} \
	  /^[a-zA-Z0-9_.-]+:.*?## / \
	  {printf "  $(BOLD)%-18s$(RESET) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@printf "\n$(DIM)Variables:$(RESET) PNPM=$(PNPM), CARGO=$(CARGO), TAURI=$(TAURI)\n"

# ---- Setup ------------------------------------------------------------------

.PHONY: install
install: ## install frontend dependencies (pnpm install)
	$(PNPM) install

.PHONY: install-frozen
install-frozen: ## install frontend dependencies using the lockfile verbatim
	$(PNPM) install --frozen-lockfile

.PHONY: deps
deps: install ## alias for install

# ---- Development ------------------------------------------------------------

.PHONY: dev
dev: ## start Tauri in development mode
	$(TAURI) dev

.PHONY: frontend-dev
frontend-dev: ## start only the Vite dev server (no Rust shell)
	$(PNPM) dev

.PHONY: preview
preview: ## preview the production frontend bundle
	$(PNPM) preview

# ---- Build ------------------------------------------------------------------

.PHONY: build
build: ## build a release Tauri bundle for the current platform
	$(TAURI) build

.PHONY: frontend-build
frontend-build: ## type-check and build the frontend bundle (no Rust bundle)
	$(PNPM) build

.PHONY: backend-build
backend-build: ## release-build the Rust binary (no installer)
	$(CARGO) build --release --manifest-path $(BACKEND_DIR)/Cargo.toml

# ---- Quality ----------------------------------------------------------------

.PHONY: check
check: ## run all fast type-checks (Rust + Vue)
	$(CARGO) check --manifest-path $(BACKEND_DIR)/Cargo.toml --all-targets
	$(PNPM) exec vue-tsc --noEmit

.PHONY: backend-check
backend-check: ## cargo check the Rust backend
	$(CARGO) check --manifest-path $(BACKEND_DIR)/Cargo.toml --all-targets

.PHONY: frontend-check
frontend-check: ## vue-tsc type-check the frontend
	$(PNPM) exec vue-tsc --noEmit

.PHONY: test
test: ## run the Rust test suite
	$(CARGO) test --manifest-path $(BACKEND_DIR)/Cargo.toml --all-targets

.PHONY: lint
lint: fmt-check clippy ## run cargo fmt --check and clippy

.PHONY: fmt
fmt: ## format Rust sources in place
	$(CARGO) fmt --manifest-path $(BACKEND_DIR)/Cargo.toml --all

.PHONY: fmt-check
fmt-check: ## verify Rust formatting without rewriting files
	$(CARGO) fmt --manifest-path $(BACKEND_DIR)/Cargo.toml --all -- --check

.PHONY: clippy
clippy: ## run clippy with warnings as errors
	$(CARGO) clippy --manifest-path $(BACKEND_DIR)/Cargo.toml --all-targets -- -D warnings

# ---- Cleanup ----------------------------------------------------------------

.PHONY: clean
clean: ## remove build artifacts (target/, dist/)
	$(CARGO) clean --manifest-path $(BACKEND_DIR)/Cargo.toml
	rm -rf dist

.PHONY: clean-all
clean-all: clean ## also remove node_modules
	rm -rf node_modules

.PHONY: reset
reset: clean-all install ## nuke build artifacts and reinstall everything

# ---- Diagnostics ------------------------------------------------------------

.PHONY: doctor
doctor: ## print toolchain versions
	@printf "$(BOLD)Toolchain$(RESET)\n"
	@printf "  node    %s\n" "$$($(NODE) --version 2>/dev/null || echo 'not installed')"
	@printf "  pnpm    %s\n" "$$($(PNPM) --version 2>/dev/null || echo 'not installed')"
	@printf "  cargo   %s\n" "$$($(CARGO) --version 2>/dev/null || echo 'not installed')"
