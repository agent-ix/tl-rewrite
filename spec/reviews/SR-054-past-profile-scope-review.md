---
id: SR-054
title: "Scope-boundary review — past-profile rewrites"
type: SpecReview
analysis: scope-boundary
scope: "FR-009 ownership across tl-rewrite, tl-syntax, tl-mltl, tl-parse, and quire-contract-ir"
review_set: all
---

## Summary

Responsibility is now explicit: tl-rewrite owns catalogued transformations,
identity-preserving reports, refusal, and replay; it consumes validated syntax
and evaluator contracts and does not own parsing, histories, native Quire
projection, shared-corpus publication, or qualification.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-5401 | high | FIXED: the shared-corpus phrase could allocate work to tl-rewrite before the corpus owner existed. FR-009 now marks tl-syntax Task-005 as owner and this implementation as its predecessor/consumer. | FR-009, tl-syntax/Task-005 |
| FND-5402 | medium | FIXED: the future-only equivalence API boundary is explicit; it returns unsupported for past and cannot stand in for the tl-mltl history evaluator or qualify a monitor. | FR-009-AC-5, TC-051 |
| FND-5403 | low | External dependencies are guaranteed at the consumed boundary by exact revision pins, structural validation, and contract/property tests. Foreign parsers, monitors, native-source authority, Quire/Quoin execution, and qualification remain out of scope. | FR-009 Inputs, Compatibility and Dependency Policy |
