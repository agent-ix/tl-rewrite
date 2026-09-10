---
id: Task-002
title: "Bind the hosted ix-flow identity"
type: Task
status: done
track: Assurance
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/NFR-003
    type: references
  - target: ix://agent-ix/tl-rewrite/TC-039
    type: verifies
---
# Task-002: Bind the hosted ix-flow identity

## Scope

Change the manual hosted workflow's executable installation to exactly
`@agent-ix/ix-flow@0.0.4` once. Add a Rust census that ignores YAML comments,
rejects every unscoped, alias, or duplicate executable spelling, requires
`workflow_dispatch` to be the sole trigger, and observes the exact released
local runtime version.

## Tests First

TC-039 must fail for an unscoped replacement, an executable alias duplicate,
and an automatic trigger. A comment-only duplicate is a required positive
control and must remain accepted.

## Completion Evidence

The focused control and mutation probes pass after Task-001. No hosted workflow
run is dispatched.

## Completion Record

Completed on 2026-09-09 after Task-001. The workflow installs exactly the scoped
`@agent-ix/ix-flow@0.0.4` package and retains its sole manual trigger. TC-039
passed the real workflow, ignored a comment-only spelling, rejected an unscoped
replacement and alias duplicate, rejected an added `push` trigger, and observed
local `ix-flow --version` as exactly `0.0.4`. Hosted CI was not dispatched.
