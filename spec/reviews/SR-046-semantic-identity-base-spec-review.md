---
id: SR-046
title: "Base specification review — semantic rewrite identity"
type: SpecReview
analysis: base
scope: "FR-002, TM-001"
review_set: base
---

# Base specification review — semantic rewrite identity

## Summary

The owner selected the base review set for the semantic interning and
formula-identity amendment. The review checked EARS grammar, identifier
integrity, matrix linkage, trace tags, and scope containment. The amendment
preserves diagnostic spans while requiring that they not affect structural
interning, formula-level identity, or cycle fingerprints.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1501 | low | No base-review defect found: FR-002-AC-4 resolves to fresh TC-040, which distinguishes parser-shaped span offsets while asserting both rule application and formula-level identities. | FR-002-AC-4, TC-040, tests/rewrite.rs |
