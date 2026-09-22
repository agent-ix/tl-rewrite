---
id: Task-003
title: "Per-gate completion records across the 13 ci prerequisites"
type: Task
status: done
track: B
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/NFR-004
    type: references
  - target: ix://agent-ix/tl-rewrite/TC-059
    type: verifies
---
# Task-003: Per-gate completion records across the 13 ci prerequisites

## Scope

Every `ci` prerequisite recipe (`fmt-check`, `lint`, `test`, `check-corpus`,
`conformance`, `counterexamples`, `normalization`, `deny`, `audit-unsafe`,
`spec`, `msrv`, `rustdoc`, `assurance`) writes a completion record — naming
itself and its own recipe's exit status — only on its own successful
completion, using the file contract Task-001 defines.

## Subtasks

- [ ] Add one shared Make function or variable (e.g. a recipe-suffix
  pattern) that every one of the 13 recipes calls/appends to write its
  record, rather than 13 independently hand-written record-writing lines.
- [ ] Apply it to all 13 recipes without changing any recipe's actual
  command or behavior — this task adds a record write, nothing else.
- [ ] Confirm a recipe that fails (e.g. a stubbed tool forced to exit
  non-zero) writes no record for that gate.
- [ ] Confirm `assurance`, which aggregates `pins` and `assurance-chain`,
  records correctly for the aggregate the way NFR-003's own measurement
  described it (an aggregate whose sub-targets can each fail independently).

## Deliverables

- Record-writing wired into all 13 `ci` prerequisite recipes via one shared
  mechanism.
- TC-059 test and fixtures, green.

## Notes

- SR-075/FND-001: this is the widest-blast-radius task in the plan — it
  touches recipes nominally owned by FR-001 through FR-010 and NFR-001
  through NFR-003. NFR-004 does not own those recipes' correctness (see
  NFR-004 Scope); this task must be reviewable as "added one record write
  per recipe" and nothing more. If a recipe's actual command needs to change
  to make this work, stop and flag it rather than quietly changing gate
  behavior under an NFR-004 task.
- Does not block or get blocked by Task-002; both depend only on Task-001.
- Unblocks: Task-004 (reconciliation needs real records to reconcile
  against), Task-006 (integration gate).
