---
id: Task-006
title: "Integration gate: tracked-measurement reproduction + clean pass"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/Task-002
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/Task-004
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/Task-005
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/NFR-004
    type: references
  - target: ix://agent-ix/tl-rewrite/TC-062
    type: verifies
  - target: ix://agent-ix/tl-rewrite/TC-063
    type: verifies
---
# Task-006: Integration gate

## Scope

Exercise the fully assembled entry point (static inspection + environment
control + completion records + reconciliation) against the exact tracked
reproduction from Linear TL-64 / `agent-ix/tl-rewrite#11`, and against a
clean positive control, together — not unit-by-unit. This is a quality gate:
do not proceed to Task-007 until both pass.

## Subtasks

- [ ] TC-062: run the entry point against a `.IGNORE:`-prepended copy of the
  real Makefile; require non-zero exit and a named violation. Repeat against
  a skeleton Makefile with every recipe replaced by a failing stub; require
  the same.
- [ ] TC-063: run the entry point against the real, unmodified Makefile in a
  clean environment with every real gate passing; require zero exit and no
  violation reported.
- [ ] Run the entry point for real against this repository's actual HEAD
  (not a fixture) and confirm it matches a bare `make ci`'s result on the
  same revision (both pass, or both fail for the same reason) — this is the
  no-false-rejection check against real work, not just fixtures.
- [ ] If either tracked-reproduction case unexpectedly passes, or the clean
  control unexpectedly fails, bisect to Task-001/002/003/004 rather than
  patching this task directly — this gate verifies integration, it does not
  own any individual mechanism's correctness.

## Deliverables

- TC-062 and TC-063 tests and fixtures, green.
- A recorded run of the entry point against real HEAD, both passing and
  (temporarily, then reverted) against a deliberately broken prerequisite,
  as the plan's own closing verification evidence.

## Notes

- This is the gate NFR-004 exists to be able to pass where the old, removed
  guard and a bare `make ci` could not: see NFR-003's Scope section for the
  exact reproduction command this task's fixtures reuse.
- Do not mark this task done on a green run against fixtures alone; the real
  HEAD run is required because fixtures cannot catch an entry-point bug that
  only manifests against the actual 13-recipe Makefile (e.g. an
  `assurance`-aggregate edge case Task-003 introduced).
- Unblocks: Task-007.
