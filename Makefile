# =============================================================================
# TL Rewrite Makefile
# =============================================================================
#
# Native orchestration. Every target calls the toolchain that owns the job:
# cargo for the crate, the rule-conformance, counterexample and normalization
# runners for the rewrite domain, quire for static export, quoin for evidence.
# Nothing here computes a verdict, attests to its own correctness, or retains
# evidence of its own.
#
# This file is not a trust root and no longer tries to be one. The parse-time
# guards that used to police Make's own execution controls — SHELL, .SHELLFLAGS,
# MAKEFLAGS, .ONESHELL, .IGNORE, the `-` prefix, $(eval), include — went with the
# collector they were protecting.
#
# Read this before trusting a green `make ci`. Measured on this file, not
# inherited from a sibling, with the command named so it can be re-derived:
#
#   make ci CARGO=false PYTHON=false QUIRE=false QUOIN=false \
#     ASSURANCE_DIR=target/ig-probe ASSURANCE_PYTHON=/bin/false
#
# exits 2 and stops at the first prerequisite. Prepend a single `.IGNORE:`
# line and the identical command exits 0 after 28 ignored recipe failures,
# with all 13 `ci` prerequisites reporting success: eleven whose own recipe
# failed, `assurance` whose three sub-targets each failed, and `audit-unsafe`,
# which invokes bash directly and the sabotage does not reach.
# The structural backstop only goes so far — Quoin binds
# each retained input by digest and the chain derives every attested result from
# the producer's own bytes, so a producer that did not run yields an absent or
# empty input that the chain names. That covers the work re-run inside
# `assurance-inputs`. It does not cover fmt-check, lint, test, check-corpus,
# deny, audit-unsafe, rustdoc, or the `quire validate` half of spec, which are
# simply neutered. Tracked as agent-ix/tl-rewrite#11.

CARGO ?= cargo
PYTHON ?= python3
QUIRE ?= quire
QUOIN ?= quoin

# The shared-assurance lane runs in its own interpreter. There is no jsonschema
# conflict left to resolve here — every script in this repository that imported
# jsonschema was part of the local evidence machinery this migration removed. The
# environment exists because engineering-assurance is pinned as a git tag, and
# resolving a git dependency into the system interpreter would make the pin
# depend on whatever else that interpreter happens to have.
ASSURANCE_VENV ?= .venv-assurance
ASSURANCE_PYTHON ?= $(ASSURANCE_VENV)/bin/python

ASSURANCE_DIR := target/assurance
RULE_RESULT := $(ASSURANCE_DIR)/rule-conformance.jsonl
COUNTEREXAMPLE_RESULT := $(ASSURANCE_DIR)/counterexample-evidence.jsonl
NORMALIZATION_RESULT := $(ASSURANCE_DIR)/normalization-sweep.jsonl
PROVENANCE_RESULT := $(ASSURANCE_DIR)/provenance-integrity.json
QUIRE_EXPORT := $(ASSURANCE_DIR)/quire-static-export.json
COMPAT_RESULT := $(ASSURANCE_DIR)/legacy-compatibility.json
MSRV_RESULT := $(ASSURANCE_DIR)/msrv.jsonl
RULE_MANIFEST := corpus/rules/manifest.json
COUNTEREXAMPLE_MANIFEST := corpus/counterexamples/manifest.json
REVISION ?= $(shell git rev-parse HEAD)

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test plus the shared-assurance tests"
	@echo "  make check-corpus     - Re-derive corpus, oracle, and dependency provenance"
	@echo "  make conformance      - Replay every catalog rule through engine and oracle"
	@echo "  make counterexamples  - Produce and replay the retained counterexample corpus"
	@echo "  make normalization    - Sweep determinism, fixed point, replay, and budgets"
	@echo "  make deny             - cargo deny check advisories, bans, licenses, sources"
	@echo "  make audit-unsafe     - Enforce // SAFETY: comments on unsafe blocks"
	@echo "  make spec             - Validate specification and coverage with Quire"
	@echo "  make msrv             - Check all targets and features with Rust 1.75"
	@echo "  make rustdoc          - Build warning-free public documentation"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean and drop the assurance environment"
	@echo "  make assurance-env    - Create the pinned shared-assurance interpreter"
	@echo "  make assurance-inputs - Run the producers and write their structured results"
	@echo "  make pins             - Classify the toolchain through the shared matrix"
	@echo "  make compat-view      - Read retained evidence through the shared mapping"
	@echo "  make assurance-chain  - Seal, retain, and verify through Quoin"
	@echo "  make assurance        - pins + compat-view + assurance-chain"
	@echo "  make ci               - All CI gates locally (hosted CI is manual-only)"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check

.PHONY: lint
lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

# The traced tests invoke the assurance gates, so the producers must already have
# run. They are a prerequisite rather than something a test creates for itself: a
# test that can produce its own inputs can produce a green run out of nothing.
.PHONY: test
test: assurance-inputs
	$(CARGO) test --all-targets --all-features

# =============================================================================
# Rewrite domain
# =============================================================================

.PHONY: check-corpus
check-corpus:
	$(PYTHON) scripts/check_provenance.py

.PHONY: conformance
conformance:
	$(CARGO) run --quiet --example rule_conformance -- --manifest $(RULE_MANIFEST)

.PHONY: counterexamples
counterexamples:
	$(CARGO) run --quiet --example counterexample_evidence -- \
		--manifest $(COUNTEREXAMPLE_MANIFEST)

.PHONY: normalization
normalization:
	$(CARGO) run --quiet --release --example normalization_sweep

.PHONY: build
build:
	$(CARGO) build --release

.PHONY: clean
clean:
	$(CARGO) clean
	rm -rf $(ASSURANCE_VENV)

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny
deny:
	$(CARGO) deny check advisories
	$(CARGO) deny check bans
	$(CARGO) deny check licenses
	$(CARGO) deny check sources

.PHONY: audit-unsafe
audit-unsafe:
	bash scripts/check_unsafe_comments.sh

.PHONY: spec
spec:
	$(QUIRE) validate --scope . 'spec/**/*.md' 'docs/*.md' --strict --summary
	$(QUIRE) coverage --scope . --strict

.PHONY: msrv
msrv:
	rustup run 1.75.0 $(CARGO) check --locked --all-targets --all-features

.PHONY: rustdoc
rustdoc:
	RUSTDOCFLAGS=-Dwarnings $(CARGO) doc --no-deps --all-features

# =============================================================================
# Shared assurance
# =============================================================================

# Rebuilt when the pin changes. Without this prerequisite, editing the pinned
# release never rebuilds the environment and the toolchain keeps whatever it
# already had.
$(ASSURANCE_PYTHON): requirements-assurance.txt
	rm -rf $(ASSURANCE_VENV)
	$(PYTHON) -m venv $(ASSURANCE_VENV)
	$(ASSURANCE_VENV)/bin/pip install --quiet --disable-pip-version-check \
		-r requirements-assurance.txt

.PHONY: assurance-env
assurance-env: $(ASSURANCE_PYTHON)

# The only target that runs a producer. Everything downstream consumes these
# files and refuses to create them.
.PHONY: assurance-inputs
assurance-inputs: assurance-env
	mkdir -p $(ASSURANCE_DIR)
	$(CARGO) run --quiet --example rule_conformance -- \
		--manifest $(RULE_MANIFEST) > $(RULE_RESULT)
	$(CARGO) run --quiet --example counterexample_evidence -- \
		--manifest $(COUNTEREXAMPLE_MANIFEST) > $(COUNTEREXAMPLE_RESULT)
	$(CARGO) run --quiet --release --example normalization_sweep > $(NORMALIZATION_RESULT)
	$(PYTHON) scripts/check_provenance.py --json > $(PROVENANCE_RESULT)
	$(QUIRE) coverage --scope . --json > $(QUIRE_EXPORT)
	$(ASSURANCE_PYTHON) scripts/legacy_evidence_view.py --json > $(COMPAT_RESULT)
	rustup run 1.75.0 $(CARGO) check --locked --all-targets --all-features \
		--message-format=json > $(MSRV_RESULT)

.PHONY: pins
pins: assurance-env
	$(ASSURANCE_PYTHON) scripts/check_shared_pins.py

.PHONY: compat-view
compat-view: assurance-env
	$(ASSURANCE_PYTHON) scripts/legacy_evidence_view.py
	$(ASSURANCE_PYTHON) scripts/legacy_evidence_view.py --mutation-probes

.PHONY: assurance-chain
assurance-chain: assurance-inputs
	$(PYTHON) scripts/assurance_chain.py --candidate-revision $(REVISION)

.PHONY: assurance
assurance: pins compat-view assurance-chain

# An operator target, not a CI gate. It writes into this repository's own Quoin
# evidence store, which is a reviewed change to spec/evidence/ rather than
# something a gate should do on every run.
.PHONY: assurance-record
assurance-record: assurance-inputs
	$(PYTHON) scripts/assurance_chain.py --adapt $(RULE_RESULT) \
		> $(ASSURANCE_DIR)/entries.json
	$(QUOIN) evidence record \
		--repo . \
		--suite SUITE-001 \
		--commit $(REVISION) \
		--tool "tl-rewrite-rule-conformance 0.1.0" \
		--adapter entries \
		--kind Integration \
		--results $(ASSURANCE_DIR)/entries.json

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test check-corpus conformance counterexamples normalization \
	deny audit-unsafe spec msrv rustdoc assurance
