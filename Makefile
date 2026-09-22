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
# This file is not a trust root and does not try to be one. The parse-time
# guards that used to police Make's own execution controls — SHELL, .SHELLFLAGS,
# MAKEFLAGS, .ONESHELL, .IGNORE, the `-` prefix, $(eval), include — went with the
# collector they were protecting, and re-adding a Make-parsing guard was
# explicitly not the remediation (it had itself accumulated findings across
# three review rounds; see NFR-004's Rationale).
#
# `make ci` alone still trusts Make's own execution controls and Make's own
# exit code, exactly as measured below. Nothing in this Makefile changed that.
# What changed is that `make ci` is no longer the assured entry point: run
# `make guarded-ci` instead. It wraps `make ci` with a Rust program, external
# to Make, that (1) refuses to invoke Make at all if this file's text — or any
# file it `include`s — carries an execution-control surface capable of
# suppressing prerequisite-failure propagation, or if the calling
# environment's MAKEFLAGS carries the same suppression; and (2), after Make
# returns, reconciles the set of gates that actually wrote a completion
# record against the declared `ci` prerequisite set below, independent of
# Make's own exit code. See `spec/requirements/NFR-004-gate-set-integrity.md`
# and `src/ci_guard.rs`. `make ci` remains directly invocable for local
# convenience; a person who runs it directly instead of `make guarded-ci`
# bypasses the binding, and that residual is disclosed rather than solved by
# removing the convenience — see NFR-004's Scope.
#
# The reproduction NFR-004 remediates, still true of `make ci` alone and no
# longer true of `make guarded-ci`, measured on this file with the command
# named so it can be re-derived:
#
#   make ci CARGO=false PYTHON=false QUIRE=false QUOIN=false \
#     ASSURANCE_DIR=target/ig-probe ASSURANCE_PYTHON=/bin/false
#
# exits 2 and stops at the first prerequisite. Prepend a single `.IGNORE:`
# line and the identical command exits 0 after 25 ignored recipe failures,
# with all 13 `ci` prerequisites reporting success: eleven whose own recipe
# failed, `assurance` whose two sub-targets each failed, and `audit-unsafe`,
# which invokes bash directly and the sabotage does not reach. `make
# guarded-ci` against the same `.IGNORE:`-prepended file refuses before Make
# ever runs.
#
# That was 28 before agent-ix/tl-rewrite#13. The figure fell by exactly three
# because the deletion removed three sabotaged recipe lines — the deleted
# compatibility target's
# two and the compatibility view's line inside `assurance-inputs` — and
# `assurance` went from three sub-targets to two. Nothing was fixed; there is
# simply less to neuter. The number is re-measured with the same command rather
# than carried forward.
# The structural backstop only goes so far — Quoin binds
# each retained input by digest and the chain derives every attested result from
# the producer's own bytes, so a producer that did not run yields an absent or
# empty input that the chain names. That covers the work re-run inside
# `assurance-inputs`. It does not cover fmt-check, lint, test, check-corpus,
# deny, audit-unsafe, rustdoc, or the `quire validate` half of spec — `make
# guarded-ci`'s completion-record reconciliation covers exactly that residue.
# Tracked as agent-ix/tl-rewrite#11.

CARGO ?= cargo
PYTHON ?= python3
QUIRE ?= quire
QUOIN ?= quoin

# `ci_guard record` is the last step of every `ci` prerequisite's recipe, so
# it only runs on that recipe's own success. Run alone (e.g. `make lint` for
# local iteration, outside `make guarded-ci`), it is a deliberate no-op.
CI_GUARD ?= $(CARGO) run --quiet --bin ci_guard --

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
	@echo "  make msrv             - Check all targets and features with Rust 1.98.1"
	@echo "  make rustdoc          - Build warning-free public documentation"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean and drop the assurance environment"
	@echo "  make assurance-env    - Create the pinned shared-assurance interpreter"
	@echo "  make assurance-inputs - Run the producers and write their structured results"
	@echo "  make pins             - Classify the toolchain through the shared matrix"
	@echo "  make assurance-chain  - Seal, retain, and verify through Quoin"
	@echo "  make assurance        - pins + assurance-chain"
	@echo "  make ci               - All CI gates locally, unguarded (see Makefile header)"
	@echo "  make guarded-ci       - The assured entry point: run this, not 'make ci'"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check
	$(CI_GUARD) record fmt-check

.PHONY: lint
lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings
	$(CI_GUARD) record lint

# The traced tests invoke the assurance gates, so the producers must already have
# run. They are a prerequisite rather than something a test creates for itself: a
# test that can produce its own inputs can produce a green run out of nothing.
.PHONY: test
test: assurance-inputs
	$(CARGO) test --all-targets --all-features
	$(CI_GUARD) record test

# =============================================================================
# Rewrite domain
# =============================================================================

.PHONY: check-corpus
check-corpus:
	$(PYTHON) scripts/check_provenance.py
	sha256sum --check corpus/past-history/SHA256SUMS
	$(CI_GUARD) record check-corpus

.PHONY: conformance
conformance:
	$(CARGO) run --quiet --example rule_conformance -- --manifest $(RULE_MANIFEST)
	$(CI_GUARD) record conformance

.PHONY: counterexamples
counterexamples:
	$(CARGO) run --quiet --example counterexample_evidence -- \
		--manifest $(COUNTEREXAMPLE_MANIFEST)
	$(CI_GUARD) record counterexamples

.PHONY: normalization
normalization:
	$(CARGO) run --quiet --release --example normalization_sweep
	$(CI_GUARD) record normalization

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
	$(CI_GUARD) record deny

.PHONY: audit-unsafe
audit-unsafe:
	bash scripts/check_unsafe_comments.sh
	$(CI_GUARD) record audit-unsafe

.PHONY: spec
spec:
	$(QUIRE) validate --scope . 'spec/**/*.md' 'docs/*.md' --strict --summary
	$(QUIRE) coverage --scope . --strict
	$(CI_GUARD) record spec

.PHONY: msrv
msrv:
	rustup run 1.98.1 $(CARGO) check --locked --all-targets --all-features
	$(CI_GUARD) record msrv

.PHONY: rustdoc
rustdoc:
	RUSTDOCFLAGS=-Dwarnings $(CARGO) doc --no-deps --all-features
	$(CI_GUARD) record rustdoc

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
	rustup run 1.98.1 $(CARGO) check --locked --all-targets --all-features \
		--message-format=json > $(MSRV_RESULT)

.PHONY: pins
pins: assurance-env
	$(ASSURANCE_PYTHON) scripts/check_shared_pins.py

.PHONY: assurance-chain
assurance-chain: assurance-inputs
	$(PYTHON) scripts/assurance_chain.py --candidate-revision $(REVISION)

.PHONY: assurance
assurance: pins assurance-chain
	$(CI_GUARD) record assurance

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

# The assured entry point (NFR-004). Builds and runs the guard, which refuses
# to invoke `make ci` at all if this file's execution controls or the calling
# environment's MAKEFLAGS could suppress a prerequisite's failure, then
# reconciles the gates that actually completed against the declared list
# above regardless of Make's own exit code. `make ci` alone still does
# neither of those — see the header comment.
.PHONY: guarded-ci
guarded-ci:
	$(CI_GUARD) ci
