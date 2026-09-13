---
id: SR-058
title: "Gap analysis — complete past-profile rewrite task"
type: SpecReview
analysis: gap-analysis
scope: "FR-009, TM-001 TC-046 through TC-052, central PLAN-010 Task-004, changed production/test/documentation paths"
review_set: all
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-009
    type: reviews
  - target: ix://agent-ix/tl-syntax/Task-004
    type: reviews
---

## Summary

Audited requirement-to-test-to-code and reverse code-to-requirement ownership
for the entire tl-rewrite Task-004 allocation. All six FR-009 criteria and all
seven local test cases are implemented; the central shared-corpus allocation is
correctly left to dependent Task-005 rather than falsely claimed here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-5801 | high | FIXED: new public past catalog, schema-preserving rebuild, temporal folds, and profile-aware report identity initially had no local owning requirement. FR-009 and TM-001 now own every changed behavior and compatibility boundary. | FR-009, TC-046..TC-052 |
| FND-5802 | medium | FIXED: all admitted past catalog entries now have discriminating semantic coverage—24 Boolean rules over complete Boolean valuations and both clocks, plus 192 generated cases for each temporal fold over every anchor. | FR-009-AC-3, FR-009-AC-6, TC-048, TC-052 |
| FND-5803 | medium | FIXED: reverse inventory found no owner for the past-catalog selection in future-only conformance refusals; FR-009-AC-5 and TC-051 now bind that non-conclusive result to the correct catalog without claiming proof. | FR-009-AC-5, TC-051 |
| FND-5804 | low | Reconciliation is complete: Quire reports 119/119 backed rows and 78/78/78 Rust bindings/tagged/candidates, with no untracked new test symbol or source/test stub. The installed module's pre-existing `Status`/`Coverage Status` structural assertion conflict remains separately known. | TM-001 |
| FND-5805 | low | No Task-004 deliverable is deferred. Canonical corpus publication/checksum and exact consumer replay are an explicit successor deliverable of central Task-005, which depends on this completed implementation. | central Task-004, central Task-005 |
