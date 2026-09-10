---
id: Task-001
title: "Implement executable package scanning"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/NFR-003
    type: references
  - target: ix://agent-ix/tl-rewrite/TC-039
    type: verifies
---
# Task-001: Implement executable package scanning

## Scope

Replace whole-workflow token counting with a focused Rust scanner that extracts
YAML `run` scripts, identifies package arguments consumed by npm `install`, `i`,
and `add`, classifies literal ix-flow specifications, and fails closed on dynamic
or unsupported package-token shapes.

## Subtasks

- [x] Extract inline and block `run` scripts, including quoted keys, without
  treating YAML metadata or word-boundary shell comments as executable.
- [x] Tokenize the required shell-command subset, retaining whether each
  package argument is literal and statically classifiable.
- [x] Census scoped/unscoped registry, alias, git/GitHub, URL, tarball/file,
  workspace/link, unversioned, and duplicate ix-flow specifications.
- [x] Extend TC-039 with positive inert-text controls and negative family,
  dynamic-expression, duplicate, trigger, and runtime controls.

## Deliverables

- Rust-only scanner and focused TC-039 coverage in `tests/shared_assurance.rs`.
- Matrix status and assurance declarations reconciled with the implemented
  criteria.

## Notes

- Production source and public APIs are outside scope.
- Hosted CI remains undispatched.
- Completion unblocks Task-002.

## Completion Record

Completed on 2026-09-09. The Rust scanner now isolates inline and block YAML
`run` scripts, tokenizes the required shell subset, inspects package arguments
to npm `install`, `i`, and `add`, and fails closed on dynamic arguments. TC-039 is
green for the exact workflow, its inert metadata/comment controls, and the
supported short npm spelling; its alternate-family, duplicate, non-literal,
trigger, and runtime controls are retained for Task-002 falsification.
