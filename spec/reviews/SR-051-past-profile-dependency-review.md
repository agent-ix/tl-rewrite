---
id: SR-051
title: "Dependency review — past-profile rewrites"
type: SpecReview
analysis: dependency
scope: "FR-009 local and central PLAN-010 Task-001 through Task-005 dependency boundary"
review_set: all
---

## Summary

The dependency graph is acyclic: accepted syntax/profile enablement and the
existing rewrite engine precede FR-009; FR-009 then precedes publication and
consumer replay of the shared corpus. Native Quire projection remains on its
separate successor track.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-5101 | high | FIXED: the local feature now names merged tl-syntax/tl-mltl contracts and FR-001/FR-002/FR-003/FR-007 as hard prerequisites rather than treating their APIs as implicit assumptions. | FR-009 Dependencies |
| FND-5102 | high | FIXED: “corpus replay” was ambiguous at the Task-004 boundary even though the canonical corpus task depends on Task-004. FR-009 explicitly assigns shared corpus publication/checksum/consumer replay to central Task-005 and limits this task to local constructed/generated cases. | FR-009 Compatibility and Dependency Policy, tl-syntax/Task-005 |
| FND-5103 | low | Verified topological order: tl-syntax#53 → tl-rewrite#38; tl-syntax#53 → tl-mltl#63 → tl-rewrite dependency pin; tl-rewrite#38 plus parser/evaluator tasks → tl-syntax#54. No cycle remains. | FR-009, central PLAN-010 |
