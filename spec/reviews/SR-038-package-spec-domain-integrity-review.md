---
id: SR-038
title: "Integrity review of the hosted ix-flow package-specification domain"
type: SpecReview
analysis: integrity
scope: "NFR-003-AC-7 through NFR-003-AC-12 and TC-039 for tl-rewrite issue 33"
review_set: all
---

## Summary

The integrity review checked completeness, consistency, atomicity, and
testability against the owning NFR and matrix. After the base-review repairs,
each package, metadata, trigger, runtime, and classification outcome has one
interpretation and an explicit TC-039 oracle.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-3801 | medium | **FIXED:** the first draft was not atomic and obscured which failure belonged to which outcome; AC-7 through AC-12 now separate the six obligations while TC-039 integrates their probes. | NFR-003-AC-7, NFR-003-AC-8, NFR-003-AC-9, NFR-003-AC-10, NFR-003-AC-11, NFR-003-AC-12 |
| FND-3802 | low | The matrix truthfully marks TC-039 as a planned correction: its existing symbol backs the row structurally but does not yet satisfy the expanded semantic domain. | TM-001, TC-039, tests/shared_assurance.rs |
| FND-3803 | high | **FIXED after exact-head review:** the implementation and ACs were incomplete for quoted `run` keys, shell word-internal hashes, and npm `add`. The requirement, matrix, implementation, and mutation controls now name the same complete domain. | NFR-003-AC-7, NFR-003-AC-9, NFR-003-AC-12, TC-039 |

## Traceability

NFR-003 owns the hosted workflow and Rust control; TC-039 traces to all six
criteria. The external npm executable is observed only for its exact released
version, while the repository owns static admission of its own workflow text.
