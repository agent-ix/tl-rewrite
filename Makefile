# =============================================================================
# TL Rewrite Makefile
# =============================================================================
#
# Native orchestration. Every target calls the toolchain that owns the job:
# cargo for the crate, the rule-conformance, counterexample and normalization
# runners for the rewrite domain, quire for static export.
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
#   make ci CARGO=false PYTHON=false QUIRE=false
#
# exits 2 and stops at the first prerequisite. Prepend a single `.IGNORE:`
# line and the identical command exits 0. `make guarded-ci` against the same
# `.IGNORE:`-prepended file refuses before Make ever runs.

CARGO ?= cargo
PYTHON ?= python3
QUIRE ?= quire

# `ci_guard record` is the last step of every `ci` prerequisite's recipe, so
# it only runs on that recipe's own success. Run alone (e.g. `make lint` for
# local iteration, outside `make guarded-ci`), it is a deliberate no-op.
CI_GUARD ?= $(CARGO) run --quiet --bin ci_guard --

# NFR-004-AC-9 (TL-202): `ci_guard ci` mints a fresh per-gate, per-run token
# and writes it here just before invoking Make. Target-specific `export`
# lines in the generated file below deliver each gate's token into only that
# gate's own recipe environment, so a subprocess of a *different* gate's own
# recipe (a compromised or buggy `cargo test`/`cargo clippy`/python/quire/
# quoin invocation, the concrete risk TL-202 names) cannot bind a completion
# record to a gate it did not itself run. `-include` — not `include` — is
# deliberate: this file is absent outside a `ci_guard ci` run (a bare `make
# ci`, `make lint`, or a fresh checkout that has never run `ci_guard ci`),
# where `ci_guard record` is already a no-op and Make must not error on the
# missing file. Generated fresh before every guarded run; never edit or
# commit it.
-include target/ci-gates/.gate-tokens.mk

RULE_MANIFEST := corpus/rules/manifest.json
COUNTEREXAMPLE_MANIFEST := corpus/counterexamples/manifest.json

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make mutants          - Full cargo-mutants run + kill-rate thresholds (hours, weekly)"
	@echo "  make mutants-diff     - cargo-mutants on the diff against origin/main + gate"
	@echo "  make coverage         - llvm-cov line/function/region coverage (stable) with floor"
	@echo "  make coverage-branch  - llvm-cov line + branch coverage (nightly)"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test"
	@echo "  make test-features    - cargo test under default and infinite-trace features"
	@echo "  make conformance      - Replay every catalog rule through engine and oracle"
	@echo "  make counterexamples  - Produce and replay the retained counterexample corpus"
	@echo "  make normalization    - Sweep determinism, fixed point, replay, and budgets"
	@echo "  make deny             - cargo deny check advisories, bans, licenses, sources"
	@echo "  make spec             - Validate specification and coverage with Quire"
	@echo "  make miri             - Run the pure suites under Miri on the pinned nightly"
	@echo "  make msrv             - Check all targets and features with the MSRV toolchain"
	@echo "  make rustdoc          - Build warning-free public documentation"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
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

.PHONY: test
test:
	$(CARGO) test --all-targets --all-features
	$(CI_GUARD) record test

# Feature matrix for the test suites: default (no features) and infinite-trace
# (which enables tl-mltl/infinite-trace). `test` above covers --all-features,
# which is the infinite-trace build; both are run here so a feature-gated test
# file or API cannot silently stop compiling standalone. Gate `test-features`.
.PHONY: test-features
test-features:
	$(CARGO) test --locked --all-targets
	$(CARGO) test --locked --all-targets --features infinite-trace
	$(CARGO) test --locked --doc
	$(CARGO) test --locked --doc --features infinite-trace
	$(CI_GUARD) record test-features

# =============================================================================
# Rewrite domain
# =============================================================================

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

.PHONY: spec
spec:
	$(QUIRE) validate --scope . 'spec/**/*.md' 'docs/DER-*.md' --strict --summary
	$(QUIRE) coverage --scope . --strict
	$(CI_GUARD) record spec

# Miri runs the pure-computation suites that finish in a reasonable time: the
# library tests, the NFR-006 boundary tests, the property suite, the disposition
# mapping, the catalog and wire-record suites, and the rewrite and rule-evidence
# suites. The corpus, oracle, replay and past-history suites hash and evaluate far
# too much to interpret in a gate, so they stay native (NFR-006-AC-6). Suites that
# spawn processes or read the corpus from disk need the host and stay native.
# Property suites lower their case count under `cfg(miri)`, and the owner-value
# boundary tests substitute caller-lowered limits (and a smaller
# `MAX_MATERIALIZED_INSTANTS`) for the owner constants (NFR-006-AC-2 to
# NFR-006-AC-4). Isolation stays on, so a test that touches the host fails; the
# `ci_guard` unit tests drive Make and the file system, so they stay native.
MIRI_TOOLCHAIN ?= nightly-2026-08-21
MIRIFLAGS ?= -Zmiri-strict-provenance -Zmiri-symbolic-alignment-check -Zmiri-env-forward=PROPTEST_DISABLE_FAILURE_PERSISTENCE
MIRI_SUITES ?= --lib --test robustness --test property --test infinite_disposition --test catalog --test wire_records --test rewrite --test rule_evidence

.PHONY: miri
miri:
	PROPTEST_DISABLE_FAILURE_PERSISTENCE=1 MIRIFLAGS="$(MIRIFLAGS)" rustup run $(MIRI_TOOLCHAIN) $(CARGO) miri test $(MIRI_SUITES) -- --skip ci_guard::
	$(CI_GUARD) record miri

.PHONY: msrv
msrv:
	rustup run 1.98.1 $(CARGO) check --locked --all-targets --all-features
	$(CI_GUARD) record msrv

.PHONY: rustdoc
rustdoc:
	RUSTDOCFLAGS=-Dwarnings $(CARGO) doc --no-deps --all-features
	$(CI_GUARD) record rustdoc

# =============================================================================
# Fuzzing
# =============================================================================

# fuzz-check compiles every cargo-fuzz target and audits the fuzz package's
# dependency graph; it never starts a campaign. It needs a nightly toolchain
# and cargo-fuzz.
.PHONY: fuzz-check
fuzz-check:
	rustup run nightly cargo fuzz build
	cd fuzz && $(CARGO) deny check --config ../deny.toml
	$(CI_GUARD) record fuzz-check

# fuzz-smoke replays every checked-in corpus entry through its target once
# (no mutation, no campaign). Longer campaigns are manual: see fuzz/README.md.
.PHONY: fuzz-smoke
fuzz-smoke:
	rustup run nightly cargo fuzz list | xargs -I % rustup run nightly cargo fuzz run % fuzz/corpus/% -- -runs=0

# =============================================================================
# Verification effectiveness (TL-231, TL-234). Heavy, so deliberately outside
# `make ci`; see docs/verification-tooling.md for budgets and measured numbers.
# =============================================================================

MUTANTS_OUT ?= target/mutants
MUTANTS_JOBS ?= 4
MUTANTS_BASE ?= origin/main
COVERAGE_MIN_LINES ?= 85
COVERAGE_IGNORE = '(ci_guard)'

# The wrapper accepts cargo-mutants' "survivors found" status and judges the
# result against mutation-thresholds.toml (see scripts/run_mutants.py).
.PHONY: mutants
mutants:
	$(PYTHON) scripts/run_mutants.py --jobs $(MUTANTS_JOBS) $(MUTANTS_OUT)

.PHONY: mutants-diff
mutants-diff:
	$(PYTHON) scripts/run_mutants.py --jobs $(MUTANTS_JOBS) --diff-base $(MUTANTS_BASE) $(MUTANTS_OUT)

.PHONY: coverage
coverage:
	$(CARGO) llvm-cov --all-features --summary-only \
		--ignore-filename-regex $(COVERAGE_IGNORE) \
		--fail-under-lines $(COVERAGE_MIN_LINES)

.PHONY: coverage-branch
coverage-branch:
	$(CARGO) +nightly llvm-cov --all-features --branch --summary-only \
		--ignore-filename-regex $(COVERAGE_IGNORE) \
		--fail-under-lines $(COVERAGE_MIN_LINES)

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test test-features conformance counterexamples normalization \
	deny fuzz-check spec msrv rustdoc miri

# The assured entry point (NFR-004). Builds and runs the guard, which refuses
# to invoke `make ci` at all if this file's execution controls or the calling
# environment's MAKEFLAGS could suppress a prerequisite's failure, then
# reconciles the gates that actually completed against the declared list
# above regardless of Make's own exit code. `make ci` alone still does
# neither of those — see the header comment.
.PHONY: guarded-ci
guarded-ci:
	$(CI_GUARD) ci
