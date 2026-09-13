---
id: SR-028
title: "Integrity review of M0 assurance follow-ups"
type: SpecReview
analysis: integrity
scope: "FR-006, NFR-003, TM-001, assurance/change-assurance.json at 6a653b4"
review_set: all
---

## Summary

FR-006 owns tracked-input lifecycle, NFR-003 owns hosted identity and trigger
integrity, and TM-001 maps each new criterion to one planned test. The sealed
declaration now names every changed governed source and restates both new
obligations without creating a second local evidence format.

## Traceability Result

| Obligation | Owning requirement | Verification | Sealed source |
| --- | --- | --- | --- |
| Tracked-input normal/unwind restoration | FR-006-AC-8 | TC-038 | FR-006, NFR-003 |
| Scoped package, comment-safe census, manual trigger, runtime pin | NFR-003-AC-7 | TC-039 | NFR-003, hosted-ci, assurance-pins |

The criteria are separable, externally observable, and do not conflict with
the existing no-local-framework or human-release boundaries.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2801 | medium | **FIXED:** the initial declaration stated only FR-006-AC-8 and could seal a candidate without naming the workflow or NFR-003-AC-7; the source connection and obligation are now explicit. | NFR-003-AC-7, hosted-ci |
