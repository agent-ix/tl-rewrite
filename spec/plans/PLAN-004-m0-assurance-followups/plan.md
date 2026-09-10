---
id: PLAN-004
title: "Close M0 unwind-safety and hosted ix-flow identity gaps"
type: Plan
status: in_progress
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-006
    type: references
  - target: ix://agent-ix/tl-rewrite/NFR-003
    type: references
---
# PLAN-004: Close M0 unwind-safety and hosted ix-flow identity gaps

## Objective

Close issue #31 by making tracked shared-input restoration safe on normal and
unwinding paths, then binding the manual hosted workflow to the released scoped
ix-flow package. Preserve the boundary that hosted CI grants no release
authority and is not dispatched by this plan.

## Requirements Summary

- FR-006-AC-8 / TC-038 owns exact tracked-input byte restoration, serialization,
  forced spawn-failure unwind, and restoration-failure behavior.
- NFR-003-AC-7 / TC-039 owns comment-safe package identity census, alias and
  duplicate rejection, the sole `workflow_dispatch` trigger, and exact local
  runtime version.
- SR-026 through SR-033 record the accepted base, failure-domain, integrity,
  dependency, evidence, risk, scope, and EARS reviews.

## Dependency Graph

```text
Task-001 (TC-038 restore safety)
  -> Task-002 (TC-039 hosted identity)
     -> Task-003 (complete local gate and closing review)
```

The graph is acyclic. NFR-003 explicitly requires restoration safety before the
hosted installation changes, so there is no safe parallel implementation track.

## Test Plan

| Test | Requirement | Planned proof |
|---|---|---|
| TC-038 | FR-006-AC-8 | Exercise normal restore, force the child spawn to panic, compare exact tracked bytes, and force scratch restoration failure while preserving the original panic. |
| TC-039 | NFR-003-AC-7 | Strip YAML comments, census every ix-flow package spelling, reject unscoped/alias/duplicate mutations, require only `workflow_dispatch`, and execute the pinned local `ix-flow --version`. |

The focused tests run before the complete isolated local gate. Mutation probes
must make each load-bearing control red while the comment-only control remains
green.

## Remaining Work

1. Implement Task-001 and falsify its normal and unwinding controls.
2. Implement Task-002 only after Task-001 passes and falsify package and trigger
   mutations.
3. Run the complete exact-toolchain local gate, reconcile the matrix, perform
   closing code/gap review, and obtain independent exact-head clearance.

## Parallel Execution Summary

No implementation tasks are parallelized. Task-001 is the reviewed prerequisite
of Task-002; Task-003 consumes both. Documentation inspection that does not alter
the candidate may overlap execution, but it cannot satisfy either test.

## Task File Mapping

- [Task-001: Restore tracked inputs across unwind](./tasks/Task-001-tracked-input-unwind.md)
- [Task-002: Bind the hosted ix-flow identity](./tasks/Task-002-hosted-ix-flow-identity.md)
- [Task-003: Validate and independently clear the candidate](./tasks/Task-003-validation-and-clearance.md)

## Coordination Rules

- Keep the shared-input serialization guard alive until exact bytes are restored.
- Preserve the original panic if restoration itself fails during unwind.
- Treat workflow comments as non-executable and count all executable ix-flow
  aliases and duplicate spellings.
- Keep hosted CI `workflow_dispatch`-only and do not dispatch it.
- Authorial local reviews may record evidence but cannot grant independent merge
  clearance.

## Qualification Boundary

This plan qualifies neither ix-flow nor a consuming monitor, makes no automatic
release decision, and does not prove native-language or FRETish equivalence.
