---
id: Task-001
title: "Artifact truth and lifecycle"
type: Task
status: planned
track: Assurance
priority: P1
relationships:
  - target: ix://agent-ix/tl-rewrite/PLAN-003
    type: part_of
  - target: ix://agent-ix/tl-rewrite/FR-006
    type: references
---
# Task-001: Artifact truth and lifecycle

## Scope

Correct the authorial status of SR-012, SR-013, and SR-014. They may record
implementation, observed local execution, reviewed-head scope, and deferred
work, but may not assert independent review clearance or use authorial records
as review credit. Update their stale claims and add reviewed-head provenance
where a record describes a historical candidate.

## Completion Evidence

Every execution statement names its operator and source; only an independently
authored review may grant clearance. PLAN-003 retains typed tasks and remains
`in_progress` until implementation and independent review are complete.
