---
id: SR-029
title: "Dependency review of M0 assurance follow-ups"
type: SpecReview
analysis: dependency
scope: "FR-006 and NFR-003 at 6a653b4"
review_set: all
---

## Summary

Both requirements are assurance enablement rather than rewrite features. The
only new serial edge is FR-006-AC-8 before NFR-003-AC-7, because changing the
hosted assurance-tool installation while the tracked-input probe can strand a
mutation would violate the review carry-forward.

## Classification

| Requirement | Class | Rationale |
| --- | --- | --- |
| FR-006-AC-8 | Enablement | Makes the shared-input mutation boundary unwind-safe before another assurance change. |
| NFR-003-AC-7 | Enablement | Binds hosted assurance tooling to one scoped executable identity and manual trigger. |

## Dependency Graph

- `FR-006-AC-8 -> NFR-003-AC-7`: restoration safety must be implemented and
  verified before the hosted assurance-tool installation changes.
- `assurance/pins.json -> NFR-003-AC-7`: the installed package and runtime
  version are checked against the released pin.

The graph is acyclic. It orders TC-038 before TC-039 and leaves the rewrite
engine independent.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2901 | medium | **FIXED:** the initial checkpoint left the restore-first prerequisite only in reviewer prose; NFR-003 now has a `depends_on` edge to FR-006 and states the ordering. | FR-006-AC-8, NFR-003-AC-7 |
