---
id: PLAN-005
title: "Close the executable ix-flow package-specification domain"
type: Plan
status: done
relationships:
  - target: ix://agent-ix/tl-rewrite/NFR-003
    type: references
---
# PLAN-005: Close the executable ix-flow package-specification domain

## Objective

Close issue #33 by making TC-039 inspect only executable package arguments in
hosted-workflow `run` scripts, while rejecting every alternate or non-literal
ix-flow package specification. Keep hosted CI manual-only and undispatched.

This is an internal assurance control. It does not define or expose another
user-authored specification language: native Quire remains the sole editable
formal-clause source profile.

## Requirements Summary

### Non-Functional Requirements

- [x] **NFR-003-AC-7:** The executable ix-flow package-specification multiset is
  exactly [`@agent-ix/ix-flow@0.0.4`].
- [x] **NFR-003-AC-8:** Alternate, duplicate, and identity-bearing package-spec
  families are rejected and reported.
- [x] **NFR-003-AC-9:** Inert comments and YAML metadata do not enter the
  executable population.
- [x] **NFR-003-AC-10:** `workflow_dispatch` is the only hosted trigger.
- [x] **NFR-003-AC-11:** The released local ix-flow runtime reports `0.0.4`.
- [x] **NFR-003-AC-12:** Dynamic or otherwise unclassifiable package arguments
  fail closed.

SR-036 through SR-043 record the accepted composite specification review.

## Dependency Graph

- `Task-001 -> Task-002`
  Reason: closing review, coverage reconciliation, and the exact-toolchain gate
  must assess the completed Rust scanner and its mutation controls.
- `NFR-003-AC-7, AC-8, AC-9, AC-12 -> TC-039 scanner implementation`
  Reason: executable-scope isolation, package-family classification, inert-text
  exclusion, and fail-closed dynamic handling are one shared parsing boundary.
- `NFR-003-AC-10, AC-11 -> Task-002`
  Reason: the existing trigger and runtime controls remain mandatory regression
  gates and must pass beside the corrected scanner.

### The seams

The work attaches to `hosted_workflow_control_errors` and its helpers in
`tests/shared_assurance.rs`. Production rewrite code and public APIs remain
unchanged; the hosted workflow stays `workflow_dispatch`-only.

## Test Plan

### Integration Tests

- [x] **TC-039 executable population:** Read only YAML `run` scripts and accept
  exactly one literal `@agent-ix/ix-flow@0.0.4` package argument across npm
  `install`, `i`, and `add` spellings.
- [x] **TC-039 alternate families:** Reject unscoped, unversioned, npm-alias,
  git, GitHub shorthand, URL, tarball/file, workspace/link, and duplicate
  ix-flow specifications while naming all observations.
- [x] **TC-039 inert controls:** Show that YAML comments, word-boundary shell
  comments, and metadata-only ix-flow spellings leave the executable population
  unchanged while a word-internal shell hash remains executable content.
- [x] **TC-039 non-literal controls:** Reject shell expansion, command
  substitution, workflow interpolation, and unsupported package-token shapes.
- [x] **TC-039 retained controls:** Reject automatic triggers and observe exact
  local `ix-flow --version` output.

### Verification

- [x] Run focused TC-039 tests and in-memory mutation probes before the full
  gate.
- [x] Run strict Quire validation and coverage, the complete isolated `make ci`
  gate with Quire 0.31.0, Quoin 0.23.1, and `@agent-ix/ix-flow@0.0.4`, then
  author closing code and gap reviews.

## Remaining Work

### Track A: Critical Path (serial)

- **A1 = Task-001** Implement the executable package scanner — medium; exit:
  TC-039 accepts only the exact scoped registry package, ignores inert YAML,
  and rejects alternate or non-literal executable arguments.
- **Gate = Task-002** Validate and review the exact candidate — measures the
  complete local assurance surface; pass: every applicable gate is green,
  mutation controls fail as intended, and closing authorial reviews record no
  unresolved blocking gap.

## Parallel Execution Summary

```text
Task-001 scanner and tests -> Task-002 mutations, full gate, and reviews
```

The tasks share the same test and traceability files, so no parallel writer is
safe or useful.

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
|---|---|---|---|---|
| Task-001 | A | NFR-003 | TC-039 | done |
| Task-002 | A | NFR-003 | TC-039 | done |

## Coordination Rules

- Keep scanner and mutation logic in first-party Rust; add no Python or shell
  collector.
- Keep hosted CI `workflow_dispatch`-only and do not dispatch it.
- Do not alter production rewrite semantics or present tl-syntax as a second
  user-authored Quire language.
- Authorial reviews may close local specification and implementation findings,
  but they do not grant independent exact-head merge clearance or human release
  acceptance.

## Post-Plan Merge Gate

Plan completion requires both tasks and closing authorial reviews. Issue closure
then requires independent review of the exact pushed head and merge through the
authorized protected path. Hosted CI and the human source-release decision stay
outside this plan.
