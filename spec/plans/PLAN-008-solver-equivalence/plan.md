---
id: PLAN-008
title: "Solver-backed equivalence behind an optional feature"
type: Plan
status: in_progress
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-091
    type: references
  - target: ix://agent-ix/tl-rewrite/NFR-076
    type: references
---
# PLAN-008: Solver-backed equivalence behind an optional feature

## Objective

Implement TL-405 (tl-rewrite FR-091, NFR-076): a non-default `solver` Cargo
feature that decides closed-trace equivalence through tl-solve beyond the
enumeration ceilings of FR-004, agrees with `check_equivalence` wherever
enumeration decides, and returns a counterexample that tl-rewrite itself
re-checked with `tl_mltl::evaluate_closed`. The default build, public API,
wire records and dependency tree are unchanged.

This bundle copies tl-solve PLAN-001 Task-012 into this repository so the
deferred verifications of the tl-rewrite side are tracked where the code lives.
FR-091, NFR-076 and TM-072 (TC-975..TC-981) live on the unmerged
`TL-147-spec` branch; this plan references them by id.

## Approach

- Miter: `¬(original ↔ rewritten)` built as one tl-syntax graph (the
  rewritten nodes shifted past the original's, through the engine's
  exhaustive `remap`), solved by `tl_solve::check_satisfiable` at exactly
  `trace_length = horizon + 1`, the enumeration domain.
- `unsatisfiable` → `equivalent` with tl-solve's basis; `satisfiable` → both
  formulas re-evaluated by tl-rewrite, `mismatch` only if the verdicts differ,
  otherwise `counterexample_not_confirmed`; every other tl-solve outcome →
  `non_conclusive` with a mirrored reason.
- Tree evidence is structural (`Cargo.toml` read by a test) plus `cargo tree`
  inside tests; no recorded tree baseline file.

## Deferred Verification

Written on branch `TL-code-unbuilt` without any cargo, rustc or make run
(TL-DECISIONS-2026-11-23). Every acceptance criterion below is unverified,
including those whose tests now exist; the expected outcome is not claimed.

| Acceptance criterion | Test | Needs to run |
|---|---|---|
| FR-091-AC-1 | TC-975 (`tests/solver_feature_boundary.rs`, `tests/solver_equivalence.rs`) | `make test-features` and `make test-solver` |
| FR-091-AC-2 | TC-976 (`tests/solver_equivalence.rs`, unit test in `src/solver_equivalence.rs`) | `cargo test --features solver` |
| FR-091-AC-3 | TC-977 (`tests/solver_equivalence.rs`) | `cargo test --features solver` |
| FR-091-AC-4 | TC-978 (`tests/solver_equivalence.rs`) | `cargo test --features solver` (release-speed machine for interval 10,000) |
| FR-091-AC-5 | TC-979 (`tests/solver_feature_boundary.rs`) | `cargo test` with a refreshed `Cargo.lock` (runs `cargo tree --offline --locked`) |
| NFR-076-AC-1 | TC-980 (`tests/solver_feature_boundary.rs`) | `cargo test` with a refreshed `Cargo.lock` |
| NFR-076-AC-2 | TC-981 (`tests/solver_feature_boundary.rs`) | `make deny` on default and all-features graphs, `make guarded-ci` reconciling `test-solver` |

### Provisional pins and unverified API guesses

| Item | Where | Needs to run / replace |
|---|---|---|
| UNBUILT-PIN tl-solve `=0.1.0`, rev `0b45d0a2`, URL `https://github.com/agent-ix/tl-solve.git` (no remote exists; local branch `TL-147-code-unbuilt`) | `Cargo.toml` | push tl-solve, replace with the merged rev; `cargo tree` (TC-979) |
| UNBUILT-PIN tl-syntax rev `a80d09ed` (TL-code branch base; LTLf variants live only on unmerged TL-code) | `Cargo.toml` | replace with the merged tl-syntax rev carrying TL-414..TL-428 |
| Pin precondition (FR-091, tl-solve Task-000): tl-rewrite names tl-mltl `011a4324` (`=0.4.0`) and tl-syntax `a80d09ed`; tl-solve `0b45d0a2` names tl-mltl `482e32fb` (`=0.4.1`) and tl-syntax `9f7b1c86`. The `solver` graph therefore holds two tl-syntax and two tl-mltl packages and cannot compile | `Cargo.toml` | converge all three crates (and tl-parse, dev-only) on one tl-syntax and one tl-mltl rev, then TC-979 |
| tl-rewrite's tl-mltl kept at `=0.4.0` so the default tree is unchanged by TL-405; convergence will move it to a 0.4.1+ rev whose API tl-rewrite has not been built against | `Cargo.toml` | `make test` after convergence |
| `Cargo.lock` not refreshed (no cargo run): it lacks tl-solve, rustsat, rustsat-batsat, batsat and the new tl-syntax rev | `Cargo.lock` | `cargo generate-lockfile --offline` / `cargo update -p tl-syntax`; every `--locked` gate fails until then |
| UNVERIFIED-API tl-solve: `check_satisfiable(&SatRequest, &mut dyn SatBackend) -> ResultDocument`, `SatRequest` fields, `FrameRequest::Closed { trace_length }`, `ResultDocument` public fields, `tl_solve::wire::result::{LimitsRecord, SolverRecord}` paths, `InProcessBackend` unit struct, `SatBackend` trait shape | `src/solver_equivalence.rs`, `tests/solver_equivalence.rs` | `cargo build --features solver` (taken from unbuilt tl-solve source) |
| UNVERIFIED-API tl-solve `RecheckLimits`, `EncodeLimits`, `SolveLimits` derive `Copy`, `Eq`, `Default` (needed by `SolverConformanceOptions`) | `src/solver_equivalence.rs` | `cargo build --features solver` |
| UNVERIFIED-API tl-mltl `EvaluationError::WorkLimitExceeded { .. }` and `EvaluationLimits { max_node_evaluations, .. }` on the converged tl-mltl rev | `src/solver_equivalence.rs` | `cargo build --features solver` |
| Behaviour guess: tl-solve's in-process backend answers interval-10,000 miters in test time, and 200,000,000 re-check evaluations suffice there | `tests/solver_equivalence.rs` (TC-978) | `make test-solver`; raise or mark `#[ignore]` with a `make` target if slow |
| `cargo tree -e features -i tl-mltl` output names `infinite-trace` exactly when it is active | `tests/solver_feature_boundary.rs` (TC-979) | `cargo test` |
| `--prefix depth` subtree extraction stands in for `cargo tree -p tl-solve --features solver` (features cannot be selected on a non-member) | `tests/solver_feature_boundary.rs` | `cargo test` |
| `make lint`, `make msrv`, `make rustdoc`, `make miri` and `make mutants-diff` now cover `--all-features` including `solver` (tl-solve, rustsat, batsat under Miri and MSRV 1.98.1) | `Makefile` | `make guarded-ci` |
| rustfmt conformance of the new files (hand-formatted) | `src/solver_equivalence.rs`, `tests/solver_*.rs` | `make fmt-check` |
| TC-975..TC-981 are not yet in this branch's spec (FR-091/NFR-076/TM-072 are on `TL-147-spec`) | `spec/` | merge TL-147-spec, then `make spec` |
| Source-compat (TL-428 LTLf `NodeKind` arms) compiles only against the unmerged tl-syntax TL-code branch | `src/engine/mod.rs`, `tests/oracle_*.rs`, `tests/future_lowering_parity.rs` | `cargo build` after the tl-syntax pin is replaced |
