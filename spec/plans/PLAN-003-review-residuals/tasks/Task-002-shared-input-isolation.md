---
id: Task-002
title: "Shared-input isolation and cleanup"
type: Task
status: planned
track: Assurance
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/PLAN-003
    type: part_of
  - target: ix://agent-ix/tl-rewrite/FR-006
    type: references
---
# Task-002: Shared-input isolation and cleanup

## Scope

Serialize every test path that reads or mutates shared assurance inputs through
one private identity-safe guard token and token-taking helpers, including the
`requirements-assurance.txt` mirror mutation and shared-pin reader. Make every
scratch, producer-shim, and control-directory cleanup fail closed.

## Completion Evidence

Removing the guard from an existing stateful path leaves the test uncompilable
or fails a retained control. Failed cleanup is reported rather than discarded,
and no new runner, collector, envelope, or retention layer is introduced.
