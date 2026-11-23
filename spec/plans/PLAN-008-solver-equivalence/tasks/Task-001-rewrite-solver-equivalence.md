---
id: Task-001
title: "Solver-backed equivalence checking in tl-rewrite"
type: Task
status: in_progress
track: Integration
priority: P1
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-091
    type: references
  - target: ix://agent-ix/tl-rewrite/NFR-076
    type: references
  - target: ix://agent-ix/tl-rewrite/TC-975
    type: verifies
  - target: ix://agent-ix/tl-rewrite/TC-976
    type: verifies
  - target: ix://agent-ix/tl-rewrite/TC-977
    type: verifies
  - target: ix://agent-ix/tl-rewrite/TC-978
    type: verifies
  - target: ix://agent-ix/tl-rewrite/TC-979
    type: verifies
  - target: ix://agent-ix/tl-rewrite/TC-980
    type: verifies
  - target: ix://agent-ix/tl-rewrite/TC-981
    type: verifies
---
# Task-001: Solver-backed equivalence checking in tl-rewrite

## Scope

Copied from tl-solve PLAN-001 Task-012. Add the optional `solver` feature to
tl-rewrite (FR-091, NFR-076), keep the default dependency tree unchanged, agree
with enumeration and return re-checked counterexamples.

Ticket: TL-405. Depends on tl-solve Task-000 (pin convergence) and Task-008
(TL-401, FR-012).

## Deliverables

- `src/solver_equivalence.rs` behind `#[cfg(feature = "solver")]`.
- `tests/solver_equivalence.rs` (TC-975..TC-978) and
  `tests/solver_feature_boundary.rs` (TC-975, TC-979..TC-981).
- `make test-solver` in the `ci` prerequisite set; `deny.toml` admits tl-solve.

## Status

Written UNBUILT on branch `TL-code-unbuilt` (2026-11-23); see the plan's
Deferred Verification table.
