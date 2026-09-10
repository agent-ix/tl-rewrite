---
id: Task-003
title: "Validate and prepare the candidate"
type: Task
status: done
track: Assurance
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/Task-002
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/FR-006
    type: references
  - target: ix://agent-ix/tl-rewrite/NFR-003
    type: references
  - target: ix://agent-ix/tl-rewrite/TC-038
    type: verifies
  - target: ix://agent-ix/tl-rewrite/TC-039
    type: verifies
---
# Task-003: Validate and prepare the candidate

## Scope

Reconcile implemented matrix state and census counts, run the complete isolated
local gate with the exact released shared-assurance toolchain, and record closing
code and traceability analysis without treating authorial artifacts as
independent clearance. Prepare the branch for an exact-head pull-request review.

## Completion Evidence

Focused mutation probes are red in their intended ways, the unmodified complete
local gate is green, Quire is strict and complete, and closing authorial analyses
state all remaining limitations. Hosted CI remains undispatched. Independent
exact-head clearance and merge are post-plan gates rather than authorial task
completion evidence.

## Completion Record

Completed on 2026-09-09. The full isolated local gate passed at `6266818` with
18/18 shared-assurance tests, 89/89 backed Quire rows, 92/92 strict documents,
MSRV, Clippy, rustdoc, Cargo Deny, provenance, pins, and the complete assurance
chain green. SR-034 found no remaining code defect; SR-035 records the known
upstream status-column limitation. No hosted workflow was dispatched.
