---
id: Task-004
title: "Declared/executed reconciliation"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/Task-003
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/NFR-004
    type: references
  - target: ix://agent-ix/tl-rewrite/TC-060
    type: verifies
  - target: ix://agent-ix/tl-rewrite/TC-061
    type: verifies
---
# Task-004: Declared/executed reconciliation

## Scope

After Make returns, compare the set of completion records Task-003's recipes
wrote against the exact declared `ci:` prerequisite set parsed from the
Makefile, and report every mismatch in either direction — a declared gate
with no record, or a record for a gate not declared. Treat a missing,
removed, or backdated record as no record.

## Subtasks

- [ ] Parse the declared `ci:` prerequisite list from the Makefile (reuse
  Task-001's Makefile-parsing code rather than re-implementing it).
- [ ] After Make exits, read the completion-record directory using Task-001's
  contract.
- [ ] Compute and report the two-way set difference by name; a mismatch in
  either direction is a violation, not just a missing record.
- [ ] Treat a record whose timestamp/revision predates the run under
  evaluation as absent, not stale-but-valid.
- [ ] Treat a record removed after being written (i.e. absent at
  reconciliation time despite having been written earlier in the same run)
  as absent.
- [ ] Never trust Make's own exit code as a substitute for this comparison —
  report based on the reconciliation, even if Make itself exited 0.

## Deliverables

- Reconciliation logic wired into the entry point.
- TC-060 (mismatch reporting) and TC-061 (missing/backdated-record handling)
  tests and fixtures, green.

## Notes

- This is where the entry point's core guarantee actually lands: everything
  in Task-001/002/003 exists to make this comparison meaningful. Do not ship
  this task without both TC-060 and TC-061 — a reconciliation that only
  checks "declared gate has *a* record" without the freshness check in AC-5
  is exactly the kind of false pass NFR-004 exists to prevent.
- Depends on Task-003's real records existing (not just Task-001's contract
  documentation) so tests exercise the actual write path, not a mock.
- Unblocks: Task-006 (integration gate).
