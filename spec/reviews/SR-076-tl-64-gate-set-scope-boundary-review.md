---
id: SR-076
title: "scope-boundary review of NFR-004 (TL-64 gate-set-integrity remediation)"
type: SpecReview
analysis: scope-boundary
scope: "spec/requirements/NFR-004-gate-set-integrity.md"
review_set: all
---

## Summary

Checked NFR-004's stated boundaries against the owner-directive block in
Linear TL-64 (Rust-only for new production/qualification logic; contain
Quoin's spread rather than expanding it; a cross-repo Quoin/Engineering
Assurance control is a separate, later decision) and against what NFR-003
already owns, to make sure NFR-004 neither silently expands into forbidden
territory nor silently re-claims something NFR-003 still owns.

## Findings

| ID      | Severity | Summary                          | Refs   |
| ------- | -------- | --------------------------------- | ------ |
| FND-001 | low      | NFR-004's Scope explicitly states the mechanism is repository-local and domain-specific, and defers any cross-repo Quoin/Engineering Assurance generalization as "a separate, later decision... out of scope here" — matching the ticket's own framing of a repo-local entry point as one of two acceptable remediation paths, and matching the owner directive's instruction to contain Quoin's spread rather than expand it. Confirmed correct; no change. | NFR-004 |
| FND-002 | low      | NFR-004 does not itself state "implemented in Rust" as a requirement line — the language choice belongs to `spec-to-plan`/implementation, not the NFR, and the repository's other NFRs (NFR-001..003) likewise describe behavior rather than implementation language. Not a gap: the owner directive is a project-wide constraint already recorded in the Linear ticket and will bind whichever plan task implements this NFR; restating it here would duplicate policy into a behavioral spec. No change. | NFR-004 |
| FND-003 | low      | Boundary between NFR-004 and NFR-003 is explicit in both directions after the ripple edit: NFR-003's Scope now says Make execution-control policing "is owned by NFR-004... rather than continuing to be described here as permanently unowned," and NFR-004's Scope explains what it does *not* own (individual gate correctness, producer-input integrity). No double-ownership or ownership gap between the two. No change. | NFR-004, NFR-003 |
