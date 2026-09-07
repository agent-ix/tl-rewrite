---
id: Task-002
title: Shared dependency and contextual wire forms
type: Task
status: in_progress
track: Core
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/PLAN-003
    type: part_of
  - target: ix://agent-ix/tl-rewrite/FR-007
    type: references
  - target: ix://agent-ix/tl-syntax/FR-007
    type: depends_on
---

# Task-002: Shared dependency and contextual wire forms

## Scope

After tl-syntax#15 lands, pin that exact reviewed revision and implement the
closed v2 forms of the native rewrite, replay, and conformance report families.
Use the shared signal/context documents directly, deterministic complete-value
digests, explicit context `null`, and version-dependent strict decoding.

## Completion Evidence

TC-031 and the wire portions of TC-035 pass. V1 bytes are captured before the
report edit and remain exact afterward. No rewrite-local signal or requirement
schema exists.

## Blocker

tl-syntax#15 has landed as `6ad7499`. The rewrite implementation is stacked on
the published tl-mltl#24 pin prerequisite so both crates resolve that one syntax
type. The dependency pin must be replaced with tl-mltl's reviewed landing before
this work is submitted; it does not block native v2 implementation locally.
