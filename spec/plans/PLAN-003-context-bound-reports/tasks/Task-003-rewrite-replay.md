---
id: Task-003
title: Context-bound rewrite and replay
type: Task
status: not_started
track: Core
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/PLAN-003
    type: part_of
  - target: ix://agent-ix/tl-rewrite/FR-007
    type: references
---

# Task-003: Context-bound rewrite and replay

## Scope

Implement context-aware rewrite and replay entry points, domain-separated
request hashing, input/output formula binding, locus-specific typed refusal,
and the complete independent catalog/context mutation table. Preserve the
existing context-free functions as v1 behavior over the common execution core.

## Completion Evidence

TC-032 and TC-033 pass, including exact replay, explicit absence, cross-formula
substitution, mutations to unused catalog entries, and absence of successful
output for an unresolved binding.
