---
id: SR-019
title: "Evidence review of the complete tl-rewrite specification corpus"
type: SpecReview
analysis: evidence
scope: "spec/**/*.md"
review_set: all
---

# SR-019: Evidence review of the complete tl-rewrite specification corpus

## Summary

Quire reports complete matrix backing (83/83 rows), but the deterministic
evidence-method advisor could not run: installed Quoin 0.23.1 rejects the
installed Quire CLI despite its reported 0.31.0 version. Consequently this
review cannot claim catalog-derived agreement between authored methods and the
current method catalog.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1901 | medium | `quoin advise --json` refuses this repository because it cannot determine the installed Quire CLI version, so the required catalog-derived method/mismatch analysis is unavailable even though the authored matrix is fully backed. | TM-001, FR-001 through FR-007, NFR-001 through NFR-003 |
