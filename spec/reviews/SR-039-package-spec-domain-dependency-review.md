---
id: SR-039
title: "Dependency review of the hosted ix-flow package-specification domain"
type: SpecReview
analysis: dependency
scope: "NFR-003-AC-7 through NFR-003-AC-12 and TC-039 for tl-rewrite issue 33"
review_set: all
---

## Summary

The change is internal enablement that depends on the scoped-package and
unwind-safety work already merged by #32. Its acyclic order is specification,
review, implementation, complete local gate, closing reviews, and independent
exact-head clearance; M1 rewrite drafts remain downstream of M0 closure.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-3901 | low | No dependency cycle or missing prerequisite remains; #32 supplies the admitted package and safe shared-input baseline, while #33 only closes the scanner's package-specification domain. | NFR-003, TC-039, tl-rewrite#32, tl-rewrite#33 |
| FND-3902 | low | The exact-head #34 findings add an in-place correction and re-review before merge; they do not change the parent dependency on merged #32 or unblock downstream M1 work early. | NFR-003, TC-039, tl-rewrite#34 review |

## Dependency Order

`tl-rewrite#32 merged` → `AC-7..AC-12 + TC-039 specified` → `full review
accepted` → `Rust control corrected` → `local gate and closing analyses` →
`independent exact-head review`. This internal lane has no dependency on, and
creates no dependency for, the native Quire language redesign.
