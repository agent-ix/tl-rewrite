---
id: Task-003
title: "Dual run, deletion, and residual"
type: Task
status: done
track: Verification
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/PLAN-002
    type: part_of
  - target: ix://agent-ix/tl-rewrite/NFR-003
    type: references
---
# Task-003: Dual run, deletion, and residual

## Scope

Exercise the old and new paths against the same candidate revision, record the
result as observed rather than as parity, delete the local evidence framework in
a separate commit afterwards, and record what the deletion genuinely costs.

## Completion Evidence

The dual run is recorded in `spec/reviews/SR-006` and in the pull request. The
base was not green: at `74aa3f10` the old path's `make ci` exits 2, failing at
`verify-evidence` with *"assured evidence is not v2-qualified"*, because the
active retained record's own revision does not carry the machinery the head's
`evidence_profile.resolve_profile` requires. That is stated as observed; no green
baseline was manufactured.

The local evidence framework is deleted in the final commit of this change,
after both paths were exercised at the same revision. `tests/shared_assurance.rs`
asserts every removed path by name.

The residual is recorded rather than closed. The Make execution-control guard is
gone and the class it policed is real; the measurement is in NFR-003, AA-001,
`assurance/change-assurance.json` and the `Makefile` header, and the issue is
`agent-ix/tl-rewrite#11`.
