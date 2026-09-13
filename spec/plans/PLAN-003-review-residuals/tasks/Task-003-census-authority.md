---
id: Task-003
title: "Census authority and control ownership"
type: Task
status: done
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

## Completion Record

Completed on 2026-09-08. The unsealed `census_controls` duplicate was removed:
Quoin's strict record schema cannot seal arbitrary control metadata, so
FR-006-AC-7 and TC-029 own the executable controls while Quoin seals the
requirement and its sources. Historical-prose exemption checking now compares a
reviewed population ground truth rather than a copied predicate. The requirement
and matrix explicitly own the non-UTF-8, hostile-exemption, stable disclosure,
and literal `ci` controls, and record why active Make prose avoids the deleted
target spelling. The exact-head review remediation keeps the executable deny
predicate independent from its reviewed identity set and drives the raw-byte
scanner with a literal hostile input whose expected matches do not move with the
scanner array.
