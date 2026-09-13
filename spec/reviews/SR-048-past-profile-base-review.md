---
id: SR-048
title: "Base specification review — past-profile rewrites"
type: SpecReview
analysis: base
scope: "FR-007, FR-009, StR-001, StR-002, NFR-001, NFR-002, TM-001; central tl-syntax FR-011/FR-013 and PLAN-010 Task-004/Task-005"
review_set: all
---

## Summary

The base checklist found and resolved two task-blocking specification defects:
the new public past rewrite surface had no local owning FR, and FR-007 froze
revision-bearing report bytes across the mandatory dependency advance. FR-009
now owns the feature and TC-046 through TC-052 cover every criterion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-4801 | high | FIXED: added FR-009 with exact inputs, outputs, admitted rules, refusals, resource boundaries, dependency identities, shared-corpus successor ownership, and six fully backed criteria. | FR-009, TC-046..TC-052 |
| FND-4802 | high | FIXED: FR-007 now preserves schema/status compatibility and deterministic bytes for one dependency set while allowing a reviewed exact dependency advance to establish a truthful new baseline. | FR-007-AC-5, FR-009-AC-6, TC-035 |
| FND-4803 | low | FIXED: TC-046 initially used the undeclared compound method `Unit/Snapshot`; it now uses the catalogued `Snapshot` method. ID sequences, links, and all six coverage rules are otherwise complete. | TM-001, TC-046 |
