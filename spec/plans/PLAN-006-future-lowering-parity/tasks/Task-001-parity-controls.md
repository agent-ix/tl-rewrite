---
id: Task-001
title: "Add the lowering lane and parity controls"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-008
    type: references
  - target: ix://agent-ix/tl-rewrite/TC-041
    type: verifies
  - target: ix://agent-ix/tl-rewrite/TC-042
    type: verifies
  - target: ix://agent-ix/tl-rewrite/TC-043
    type: verifies
  - target: ix://agent-ix/tl-rewrite/TC-044
    type: verifies
  - target: ix://agent-ix/tl-rewrite/TC-045
    type: verifies
---
# Task-001: Add the lowering lane and parity controls

## Scope

Add the dev-only lowering lane and the direct-versus-lowered rewrite controls.

## Subtasks

- [x] Add `tl-syntax-lowering` (tl-syntax `8dc18ee`) and `tl-parse-derived`
  (tl-parse `9ca856b`) as renamed dev-dependencies.
- [x] Build each case twice: by hand, and through
  `FutureLoweringRequest::lower()` plus a byte-exact formula v1 wire crossing.
- [x] Compare complete rewrite, budgeted, contextual, and conformance reports.
- [x] Compare clean-ascii/v2 parses by span-insensitive identity.
- [x] Scan `src/`, `examples/`, and every production dependency section.
- [x] Add twelve lowering mutants and require each to fail parity.
- [x] Require `scripts/check_provenance.py` to tie the production tl-syntax pin
  to tl-mltl's locked revision and refuse undeclared locked revisions, with
  TC-030 probes for both.

## Deliverables

- `tests/future_lowering_parity.rs` with TC-041 through TC-045.
- FR-008, TM-001 rows, and this plan bundle.

## Notes

- Production source is unchanged.

## Completion Record

Completed on 2026-09-12. All five tests pass locally, and a hand mutation of the
direct builder turned TC-041, TC-042, TC-043, and TC-045 red.
