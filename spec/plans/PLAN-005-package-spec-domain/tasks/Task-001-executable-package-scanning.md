---
id: Task-001
title: "Implement executable package scanning"
type: Task
status: in_progress
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
YAML `run:` scripts, identifies package arguments consumed by `npm install` and
`npm i`, classifies literal ix-flow specifications, and fails closed on dynamic
or unsupported package-token shapes.

## Subtasks

- [ ] Extract inline and block `run:` scripts without treating comments or YAML
  metadata as executable.
- [ ] Tokenize the required shell-command subset, retaining whether each
  package argument is literal and statically classifiable.
- [ ] Census scoped/unscoped registry, alias, git/GitHub, URL, tarball/file,
  workspace/link, unversioned, and duplicate ix-flow specifications.
- [ ] Extend TC-039 with positive inert-text controls and negative family,
  dynamic-expression, duplicate, trigger, and runtime controls.

## Deliverables

- Rust-only scanner and focused TC-039 coverage in `tests/shared_assurance.rs`.
- Matrix status and assurance declarations reconciled with the implemented
  criteria.

## Notes

- Production source and public APIs are outside scope.
- Hosted CI remains undispatched.
- Completion unblocks Task-002.

