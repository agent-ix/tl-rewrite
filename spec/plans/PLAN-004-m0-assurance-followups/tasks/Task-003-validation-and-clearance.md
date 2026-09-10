---
id: Task-003
title: "Validate and independently clear the candidate"
type: Task
status: in_progress
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
# Task-003: Validate and independently clear the candidate

## Scope

Reconcile implemented matrix state and census counts, run the complete isolated
local gate with the exact released shared-assurance toolchain, and record closing
code and traceability review without treating authorial artifacts as independent
clearance. Open the pull request and obtain a review tied to its exact head.

## Completion Evidence

Focused mutation probes are red in their intended ways, the unmodified complete
local gate is green, Quire is strict and complete, an independent reviewer clears
the exact head, and that head merges through the authorized protected path.
Hosted CI remains undispatched.
