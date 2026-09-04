---
id: Task-004
title: Context-bound equivalence and native intake
type: Task
status: not_started
track: Interoperability
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/PLAN-003
    type: part_of
  - target: ix://agent-ix/tl-rewrite/FR-007
    type: references
  - target: ix://agent-ix/tl-rewrite/FR-006
    type: references
---

# Task-004: Context-bound equivalence and native intake

## Scope

Implement contextual bounded-equivalence reporting, pre-enumeration binding of
both operands, distinct original/rewritten refusal, and one exercised path from
an existing native producer through the existing Quoin intake boundary.

## Completion Evidence

TC-034 and TC-036 pass. The retained producer bytes contain the native
contextual result, Quoin and Quire run no producer, and the repository contains
no new generic execution, envelope, adapter framework, or retention code.
