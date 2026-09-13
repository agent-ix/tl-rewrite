---
id: SR-027
title: "Failure-domain review of M0 assurance follow-ups"
type: SpecReview
analysis: failure-domain
scope: "FR-006-AC-8, NFR-003-AC-7, TC-038, TC-039 at 6a653b4"
review_set: all
---

## Summary

The review exercised mutation setup, normal restoration, panic unwind,
restoration failure during unwind, poisoned serialization, package-alias
duplication, comment-only tokens, trigger drift, and runtime-version drift. The
amended requirements state each fail-closed outcome and leave no unresolved
failure-domain gap in this bounded change.

## Failure-Domain Result

- The guard captures bytes before the first mutation and retains the existing
  single-writer lock until restoration.
- Explicit restoration covers success; `Drop` covers panic unwind; a failed
  unwind restoration reports without starting a second panic.
- The workflow census removes comments before interpreting tokens, so comments
  neither satisfy nor violate the executable identity count.
- An added unscoped or scoped alias remains executable content and must make the
  same test red; automatic triggers and wrong runtime versions are distinct
  failures.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2701 | medium | **FIXED:** unwind restoration failure was initially stated only as “no double panic” without requiring preservation of the original panic; TC-038 now makes that state observable. | FR-006-AC-8, TC-038 |
