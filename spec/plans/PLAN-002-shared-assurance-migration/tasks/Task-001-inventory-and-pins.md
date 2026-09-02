---
id: Task-001
title: "Inventory, pins, and the blocked upstream repin"
type: Task
status: done
track: Foundation
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/PLAN-002
    type: part_of
  - target: ix://agent-ix/tl-rewrite/FR-006
    type: references
---
# Task-001: Inventory, pins, and the blocked upstream repin

## Scope

Produce the keep/replace/delete/defer inventory, declare the adopted release in
`assurance/pins.json`, delegate every version verdict to the packaged
compatibility matrix, and attempt to move the tl-syntax pin onto a revision
reachable from that repository's `main`.

## Completion Evidence

`scripts/check_shared_pins.py` classifies four components through
`engineering_assurance.compatibility` and restates no version rule locally. The
hosted workflow's `@agent-ix/quoin@0.22.5` pin — a version the matrix names
explicitly incompatible — is repinned to 0.23.1 and `ix-flow@0.0.4` is added.

The tl-syntax repin was attempted and is blocked, with the reason measured
rather than assumed: `tl-mltl` at its merged `main` `fe1c620d` still pins
`740182f1`, this crate passes `tl_syntax::Formula` values straight into
`tl_mltl::evaluate_closed`, and moving only this pin puts two `tl-syntax`
versions in the graph, which cargo reports as E0308 at four call sites in
`src/equivalence.rs`. Recorded as an open unknown and filed as
`agent-ix/tl-rewrite#10`.

A separate pin defect was found and fixed: `TL_MLTL_REVISION` disagreed with
`Cargo.toml`, so every conformance report named an evaluator revision the build
had not used. `scripts/check_provenance.py` now requires the constants,
`Cargo.toml` and `Cargo.lock` to agree, and a mutation probe restores the stale
value to show the check goes red.
