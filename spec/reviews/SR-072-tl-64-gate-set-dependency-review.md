---
id: SR-072
title: "dependency review of NFR-004 (TL-64 gate-set-integrity remediation)"
type: SpecReview
analysis: dependency
scope: "spec/requirements/NFR-004-gate-set-integrity.md"
review_set: all
---

## Summary

Checked whether NFR-004's `depends_on FR-006` relationship is a real,
justified dependency or an uncritical copy of NFR-003's identical edge, and
whether NFR-004 is enablement work or feature work relative to the ticket it
remediates.

## Findings

| ID      | Severity | Summary                          | Refs   |
| ------- | -------- | --------------------------------- | ------ |
| FND-001 | low      | `depends_on FR-006` was initially carried over from NFR-003 without restating why it applies to NFR-004 too. The real dependency is narrower than NFR-003's: NFR-004 depends on FR-006's Quoin-bound `assurance-inputs` digest chain remaining intact, because NFR-004's Scope explicitly relies on that chain to cover the four gates re-run inside `assurance-inputs` so NFR-004 does not have to duplicate them. Scope text already states this; the relationship is justified, not copied blind. No structural change needed, but noting the distinct rationale here so a future reader does not assume the two NFRs depend on FR-006 for the same reason. | NFR-004, FR-006 |
| FND-002 | low      | NFR-004 is enablement/assurance-infrastructure work, not a product feature: it changes nothing about tl-rewrite's rewrite semantics, only what is trusted about a green `make ci`. Correctly scoped as an NFR under `quality_attribute: reliability` rather than an FR: it has no user-observable behavior of the rewrite system itself. | NFR-004 |
