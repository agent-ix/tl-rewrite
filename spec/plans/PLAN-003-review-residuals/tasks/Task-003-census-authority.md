---
id: Task-003
title: "Census authority and control ownership"
type: Task
status: planned
track: Assurance
priority: P1
relationships:
  - target: ix://agent-ix/tl-rewrite/PLAN-003
    type: part_of
  - target: ix://agent-ix/tl-rewrite/FR-006
    type: references
  - target: ix://agent-ix/tl-rewrite/NFR-003
    type: references
---
# Task-003: Census authority and control ownership

## Scope

Replace historical-prose expected-side copies with authored population ground
truth or a non-tautological control. State the authority boundary of
`census_controls` and move any sealable claim through the existing shared
contract rather than a local envelope. Reconcile the active Make disclosure,
the deleted-target needle, and ownership of non-UTF-8, hostile-exemption,
disclosure-pin, and `ci`-declaration controls.

## Completion Evidence

Each retained control has one owning requirement and a falsifiable expected
side. The active disclosure records its deliberate trade without becoming a
false positive, and the shared-contract boundary remains explicit.
