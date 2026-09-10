---
id: SR-019
title: "Integrity review of the complete tl-rewrite specification corpus"
type: SpecReview
analysis: integrity
scope: "spec/**/*.md"
review_set: all
---

# SR-019: Integrity review of the complete tl-rewrite specification corpus

## Summary

The requirement set is substantially traceable to the matrix and Quire reports
83 of 83 matrix rows backed. The remaining integrity issue is that the
acceptance-criterion identifiers are not sequential in two FRs, making the
historical artifact chain ambiguous even though the listed criteria are covered.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1701 | medium | The unexplained missing `FR-005-AC-2` and `FR-006-AC-4` identifiers prevent a reviewer from distinguishing deliberate retirement from an omitted obligation without external history. | FR-005, FR-006 |
