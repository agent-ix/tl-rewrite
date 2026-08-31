# =============================================================================
# TL Rewrite Makefile
# =============================================================================

tl_make_short_flags := $(firstword $(MAKEFLAGS))
ifneq ($(findstring i,$(tl_make_short_flags)),)
$(error local CI refuses Make ignore-errors mode)
endif
ifneq ($(findstring --ignore-errors,$(MAKEFLAGS)),)
$(error local CI refuses Make ignore-errors mode)
endif

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test"
	@echo "  make check-failure-propagation - prove required command failures reach CI"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
	@echo "  make deny             - cargo deny check licenses and sources"
	@echo "  make audit-unsafe     - Enforce // SAFETY: comments on unsafe blocks"
	@echo "  make check-corpus     - Verify retained WEST corpus bytes"
	@echo "  make verify-evidence  - Verify every retained evidence SHA-256 manifest"
	@echo "  make spec             - Validate and cover specification artifacts"
	@echo "  make rustdoc          - Build warning-free public documentation"
	@echo "  make msrv             - Test all targets and features with Rust 1.75"
	@echo "  make evidence-tool    - Syntax-check evidence tooling and schemas"
	@echo "  make ci               - All CI gates locally (fmt-check + lint + test + deny + audit-unsafe)"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: fmt-check
fmt-check:
	cargo fmt --all -- --check

.PHONY: lint
lint:
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: test
test:
	cargo test --all-targets --all-features

.PHONY: check-failure-propagation
check-failure-propagation:
	python3 scripts/check_failure_propagation.py

.PHONY: check-corpus
check-corpus:
	python3 scripts/check_corpus.py

.PHONY: verify-evidence
verify-evidence:
	bash scripts/verify_evidence.sh

.PHONY: spec
spec:
	quire validate --scope . 'spec/**/*.md'
	python3 scripts/check_traceability_coverage.py

.PHONY: rustdoc
rustdoc:
	RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features

.PHONY: evidence-tool
evidence-tool:
	python3 -m compileall -q scripts
	python3 scripts/run_policy_tests.py

.PHONY: build
build:
	cargo build --release

.PHONY: clean
clean:
	cargo clean

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny
deny:
	cargo deny check licenses
	cargo deny check sources

.PHONY: cargo-audit
cargo-audit:
	cargo audit

.PHONY: audit-unsafe
audit-unsafe:
	bash scripts/check_unsafe_comments.sh

.PHONY: msrv
msrv:
	cargo +1.75.0 test --all-targets --all-features

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: check-failure-propagation fmt-check lint test check-corpus deny audit-unsafe evidence-tool spec msrv rustdoc verify-evidence
