---
id: SR-053
title: "Risk and complexity review — past-profile rewrites"
type: SpecReview
analysis: risk-complexity
scope: "FR-009 with NFR-001/NFR-002 and central FR-011/FR-013"
review_set: all
---

## Summary

FR-009 has high semantic risk and low volatility: a wrong fold silently changes
monitor meaning, but the accepted upstream profile is immutable. Property,
mutation, replay, and exact-pin controls mitigate that risk before merge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-5301 | high | MITIGATED: two profile-sensitive temporal folds and pre-origin semantics are high-risk. The catalog admits only the two stated identities and TC-052 compares both against the pinned evaluator across generated boundaries and both clocks. | FR-009-AC-2, FR-009-AC-6, TC-047, TC-052 |
| FND-5302 | medium | MITIGATED: report/catalog/dependency identity coupling can create compatibility churn. The legacy catalog is digest-pinned, report baseline changes are explicit, and provenance requires compiled and reported revisions to agree. | FR-007-AC-5, FR-009-AC-1, FR-009-AC-6 |
| FND-5303 | low | Shared-corpus availability is medium schedule risk but low semantic volatility; central Task-005 is the named successor and FR-009 makes no premature completion claim. | FR-009 Compatibility and Dependency Policy |
