# =============================================================================
# TL Rewrite Makefile
# =============================================================================

ifneq ($(filter ci ci-for-evidence,$(MAKECMDGOALS)),)
ifneq ($(strip $(MAKEFLAGS)),)
$(error local CI refuses non-empty MAKEFLAGS)
endif
ifneq ($(strip $(PYTHONOPTIMIZE)),)
$(error local CI refuses optimized Python policy execution)
endif
tl_ci_static_status := $(shell /usr/bin/env -u PYTHONOPTIMIZE MAKEFLAGS= /usr/bin/python3 scripts/check_failure_propagation.py --makefile '$(firstword $(MAKEFILE_LIST))' --static-only >/dev/null; echo $$?)
ifneq ($(tl_ci_static_status),0)
$(error local CI refuses unsafe Make recipe controls)
endif
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
	@echo "  make check-tool-identities - Verify host-scoped qualification executables"
	@echo "  make ci-for-evidence  - Candidate gates before assurance self-binding"
	@echo "  make ci               - Portable complete gate, including retained evidence"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: fmt-check
fmt-check:
	cargo fmt --all -- --check
	@/usr/bin/printf 'fmt-check gate passed\n'

.PHONY: lint
lint:
	cargo clippy --all-targets --all-features -- -D warnings
	@/usr/bin/printf 'lint gate passed\n'

.PHONY: test
test:
	cargo test --all-targets --all-features
	@/usr/bin/printf 'Rust test gate passed\n'

.PHONY: check-failure-propagation
check-failure-propagation:
	/usr/bin/python3 scripts/check_failure_propagation.py

.PHONY: check-corpus
check-corpus:
	/usr/bin/python3 scripts/check_corpus.py
	@/usr/bin/printf 'corpus-integrity gate passed\n'

.PHONY: verify-evidence
verify-evidence:
	/usr/bin/bash scripts/verify_evidence.sh
	@/usr/bin/printf 'verify-evidence gate passed\n'

.PHONY: spec
spec:
	quire validate --scope . 'spec/**/*.md'
	/usr/bin/python3 scripts/check_traceability_coverage.py
	@/usr/bin/printf 'spec gate passed\n'

.PHONY: rustdoc
rustdoc:
	RUSTDOCFLAGS=-Dwarnings /usr/bin/bash scripts/check_rustdoc.sh
	@/usr/bin/printf 'rustdoc gate passed\n'

.PHONY: evidence-tool
evidence-tool:
	/usr/bin/python3 -m compileall -q scripts
	/usr/bin/python3 scripts/run_policy_tests.py
	@/usr/bin/printf 'evidence-tool gate passed\n'

.PHONY: check-tool-identities
check-tool-identities:
	/usr/bin/python3 scripts/tool_identity.py --verify-live
	@/usr/bin/printf 'qualified-tool-identities gate passed\n'

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
	cargo deny check advisories
	cargo deny check licenses
	cargo deny check sources
	@/usr/bin/printf 'deny gate passed\n'

.PHONY: cargo-audit
cargo-audit:
	cargo audit

.PHONY: audit-unsafe
audit-unsafe:
	/usr/bin/bash scripts/check_unsafe_comments.sh
	@/usr/bin/printf 'audit-unsafe gate passed\n'

.PHONY: msrv
msrv:
	cargo +1.75.0 test --all-targets --all-features
	@/usr/bin/printf 'msrv gate passed\n'

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci ci-for-evidence
ci-for-evidence: check-failure-propagation check-tool-identities fmt-check lint test check-corpus deny audit-unsafe evidence-tool spec msrv rustdoc

ci: check-failure-propagation fmt-check lint test check-corpus deny audit-unsafe evidence-tool spec msrv rustdoc verify-evidence
