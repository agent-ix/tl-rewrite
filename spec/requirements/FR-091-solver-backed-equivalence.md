---
id: FR-091
title: Decide rewrite equivalence with a solver behind an optional feature
type: FR
relationships:
  - target: ix://agent-ix/tl-rewrite/StR-002
    type: implements
  - target: ix://agent-ix/tl-rewrite/FR-004
    type: depends_on
  - target: ix://agent-ix/tl-solve/FR-012
    type: depends_on
  - target: ix://agent-ix/tl-solve/ADR-004
    type: depends_on
---

# FR-091: Decide rewrite equivalence with a solver behind an optional feature

## Description

Where the non-default `solver` Cargo feature is enabled, tl-rewrite shall decide
equivalence of a closed-trace formula pair by asking tl-solve, remove the
enumeration ceiling of [FR-004](./FR-004-equivalence.md), and return a
re-checked counterexample trace when the pair differs. With the feature off,
tl-rewrite shall keep the default build, the public API, the wire records and
the dependency tree unchanged.

## Inputs

- Two `FormulaDocument`s under `mltl.closed-trace/v1`, a comparison id, and
  `SolverConformanceOptions { limits, recheck_limits, backend }` (tl-solve
  limits, tl-solve `RecheckLimits` and one tl-solve `SatBackend`).

## Outputs

- A `SolverConformanceReport` with schema `tl-rewrite.solver-conformance/v1`: the
  formula digests, profile, trace length used, status (`equivalent`, `mismatch`,
  `non_conclusive`), a typed reason when non-conclusive, the tl-solve basis and
  solver identity when `equivalent`, and a counterexample trace with both
  verdicts when `mismatch`. The existing `ConformanceReport` and its schema
  `tl-rewrite.conformance/v1` are not modified.

## Behavior

- The feature is `solver = ["dep:tl-solve", "tl-solve/inprocess-sat"]` with
  `default = []`. tl-solve is an optional dependency with
  `default-features = false`, exact version, and an exact git revision equal to
  the one used by tl-solve's own tl-syntax and tl-mltl pins (tl-solve ADR-004,
  item 4). The feature does not enable tl-mltl's `infinite-trace`.
- The solver path calls tl-solve's equivalence check; `equivalent` is reported
  only for tl-solve's `unsatisfiable` disposition, carrying its basis
  (`encoder+solver` or `encoder+checked-proof`).
- A `mismatch` is reported only after tl-rewrite itself evaluates both formulas
  on the returned trace with `tl_mltl::evaluate_closed` and confirms the verdicts
  differ; a counterexample that fails this check is a typed failure, never a
  mismatch. That evaluation runs with the explicit `recheck_limits` (default:
  tl-mltl's evaluator default of 1,000,000 node evaluations, the same value
  tl-solve FR-009 uses), and the limits used are recorded in the report. An
  evaluator stop on that budget is `non_conclusive` with reason `recheck-limit`,
  never a mismatch; deciding a pair at intervals up to 10,000 may need a raised
  budget.
- **Pin precondition (TL-394).** The optional `tl-solve` edge is added only
  after tl-rewrite and tl-mltl resolve one tl-syntax revision and one tl-oracle
  name (today tl-rewrite names tl-syntax `9607722f` and tl-mltl `011a4324`,
  while tl-mltl main names tl-syntax `9f7b1c86`); tl-solve FR-017-AC-6 states
  the check, and FR-091-AC-5 detects a violation in the resolved graph.
- A solver limit, timeout, capability gap, tl-solve refusal or failure is
  `non_conclusive` with its reason, and is absent from positive rule-enablement
  evidence (as in FR-004-AC-2).
- The enumeration path is not changed and remains the default evidence; the
  solver path adds evidence and does not replace it.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-091-AC-1 | With `--no-default-features`, `cargo build` and `cargo test` of the default configuration compile no solver code and the `tl-rewrite.conformance/v1` outputs of the existing tests are byte-identical to their recorded values; with `--features solver` the solver path is present and the existing tests still pass. | integration-testing (TC-975) |
| FR-091-AC-2 | On every formula pair that `check_equivalence` can decide (within `ConformanceOptions::default()` and in the declared small-scope population of every enabled rule), the solver path returns `equivalent` exactly when enumeration returns `equivalent`, and `mismatch` exactly when it returns `mismatch`. | property-based-testing (TC-976) |
| FR-091-AC-3 | For every mismatch the report carries a counterexample trace on which `evaluate_closed` gives different verdicts for the two formulas, checked by the test independently of the production re-check, and a seeded tl-solve fault that returns a trace on which the verdicts agree produces a typed failure and no mismatch. | integration-testing (TC-977) |
| FR-091-AC-4 | Rule instances whose horizon or proposition count exceeds the enumeration ceilings, including intervals up to 10,000, are decided by the solver path while enumeration returns `non_conclusive`, and a conflict limit, a time limit, a re-check budget stop and a refused backend each give `non_conclusive` with a distinct reason and never `equivalent` or `mismatch`. | integration-testing (TC-978) |
| FR-091-AC-5 | The resolved graph with `--features solver` contains one tl-syntax and one tl-mltl package, contains no `infinite-trace` activation of tl-mltl unless the caller also requests tl-rewrite's own `infinite-trace`, and contains no edge from tl-solve to tl-rewrite. | integration-testing (TC-979) |

## Dependencies

Depends on FR-004 and tl-solve FR-012. NFR-076 constrains the default tree and
the gate set. tl-solve ADR-004 holds the cycle analysis.
