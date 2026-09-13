---
id: SR-017
title: "Base review of the complete tl-rewrite specification corpus"
type: SpecReview
analysis: base
scope: "spec/**/*.md"
review_set: all
---

# SR-017: Base review of the complete tl-rewrite specification corpus

## Summary

This retroactive base review read the complete specification corpus, including
requirements, assurance artifacts, plans, matrix, suites, and prior reviews.
Quire validates the corpus, but two ID-sequence/uniqueness defects remain in
the authored review and requirement artifacts.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1501 | medium | `SR-012` is assigned to both the context-bound reports review and the review-residuals composite, so SpecReview IDs are not unique. | SR-012 |
| FND-1502 | medium | FR-005 omits `FR-005-AC-2`, and FR-006 omits `FR-006-AC-4`; the corpus gives no discontinuity rationale, so the acceptance-criterion sequence is not complete. | FR-005, FR-006 |
