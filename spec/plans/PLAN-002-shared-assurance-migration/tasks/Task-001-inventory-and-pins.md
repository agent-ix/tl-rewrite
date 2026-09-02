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

Both temporal pins move onto revisions reachable from their own `main`:
`tl-syntax` `740182f1` to `953ee825`, and `tl-mltl` `fe1c620d` to `f7eb8bdf`.

The order was forced and was measured rather than assumed. The `tl-syntax` repin
was attempted first, while `tl-mltl` still pinned `740182f1`, and cargo reported
E0308 at four call sites in `src/equivalence.rs` — this crate passes
`tl_syntax::Formula` values straight into `tl_mltl::evaluate_closed`, so both
must resolve the same revision. It was reverted and carried as an open unknown
until `tl-mltl`'s own migration merged carrying its repin, after which both moved
together.

A separate pin defect was found and fixed: `TL_MLTL_REVISION` disagreed with
`Cargo.toml`, so every conformance report named an evaluator revision the build
had not used. `scripts/check_provenance.py` now requires the constants,
`Cargo.toml` and `Cargo.lock` to agree, and a mutation probe restores the stale
value to show the check goes red.
