---
id: TM-072
title: Solver-backed equivalence test matrix
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-091
    type: covers
---

# Solver-backed equivalence test matrix

All rows are planned: no code exists, and the tests need a build, the optional
tl-solve crate and a SAT backend, none of which could be run when this
specification was written (see tl-solve PLAN-001, deferred verification).

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-091 | FR-091-AC-1 through FR-091-AC-5 | TC-975 through TC-979 | 🚧 planned |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-076 | dependency-tree comparison and gate-set inspection | TC-980, TC-981 | 🚧 planned |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-975 | The default build compiles no solver code and existing conformance outputs are byte-identical, and the solver feature leaves the existing tests passing | Integration | P0 | FR-091-AC-1 | 🚧 planned, deferred verification (needs a build and test run) |
| TC-976 | The solver path agrees with enumeration on every pair that enumeration decides, over the default options and the rule catalog's small-scope population | Property | P0 | FR-091-AC-2 | 🚧 planned, deferred verification (needs a solver run) |
| TC-977 | Every mismatch carries a counterexample that evaluate_closed confirms, and a seeded solver fault becomes a typed failure | Integration | P0 | FR-091-AC-3 | 🚧 planned, deferred verification (needs a solver run) |
| TC-978 | Pairs beyond the enumeration ceilings, up to interval 10,000, are decided by the solver path and every limit or refusal is non-conclusive | Integration | P0 | FR-091-AC-4 | 🚧 planned, deferred verification (needs a solver run) |
| TC-979 | The solver-feature graph has one tl-syntax and one tl-mltl, no infinite activation, and no edge from tl-solve to tl-rewrite | Integration | P0 | FR-091-AC-5 | 🚧 planned, deferred verification (needs cargo tree) |
| TC-980 | The default dependency tree equals the recorded baseline and the solver tree differs only by tl-solve | Integration | P0 | NFR-076-AC-1 | 🚧 planned, deferred verification (needs cargo tree) |
| TC-981 | cargo deny passes on both graphs and the ci gate set declares and reconciles the solver gate | Integration | P1 | NFR-076-AC-2 | 🚧 planned, deferred verification (needs cargo-deny and make ci) |
