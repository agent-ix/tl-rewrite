---
id: SR-083
title: "scope-boundary review of NFR-004-AC-9 (TL-202 per-gate record binding)"
type: SpecReview
analysis: scope-boundary
scope: "spec/requirements/NFR-004-gate-set-integrity.md"
review_set: subset
---

## Summary

Checked AC-9 and its Scope item 4/residual paragraph against the
owner/component boundaries the base NFR-004 review round (SR-069..SR-076)
already established, and against the repository's own "focused, honest
residual disclosure" convention. No boundary conflict or scope creep found;
the residual disclosure is precise about what closes and what does not.

## Findings

| ID      | Severity | Summary                          | Refs   |
| ------- | -------- | --------------------------------- | ------ |
| FND-001 | low      | Item 4 is owned by the same single artifact items 1-3 already own — the CI entry point (`ci_guard.rs`/`src/bin/ci_guard.rs`), not a new component — and does not touch NFR-003's boundary (producer-input integrity via the Quoin-bound `assurance-inputs` chain, per SR-071/FND-003's already-recorded ownership split). No double-ownership. No change. | NFR-004-AC-9 |
| FND-002 | low      | The residual-disclosure paragraph added after item 4 names exactly one closed boundary (inheritance-only forgery via a shared `CI_GUARD_RUN_ID`) and exactly one open one (a subprocess that deliberately reads the token store off disk, which nothing in this control encrypts), and states the reason a full close is out of scope (per-gate OS-level process isolation is a materially larger architectural change) in the same terms the existing "person runs `make ci` directly" residual paragraph already uses two paragraphs earlier. Matches this document's established convention of disclosing rather than claiming a control closes more than it does. No change. | NFR-004-AC-9 |
| FND-003 | low      | External dependency: AC-9 deepens reliance on GNU Make's target-specific-variable scoping (already flagged from the integrity angle in SR-081/FND-004). From a boundary standpoint this is an **assumed**, not separately contract-tested, external dependency — the same status Make itself already has for items 1-3 (the entry point trusts Make's documented exit-code and `-include`-missing-file semantics without a contract test asserting them, e.g. verified instead by direct fixture-level integration tests such as TC-062/TC-064 exercising real `make` runs). Consistent with the existing boundary; no new dependency class. No change. | NFR-004-AC-9 |
| FND-004 | low      | Repository-locality: item 4, like items 1-3, is repository-local to this Makefile and `ci` target (per the existing "not a proposal for a shared, cross-repository control" paragraph, which the new residual paragraph explicitly reuses rather than re-litigates). No scope creep toward Quoin/Engineering Assurance. No change. | NFR-004-AC-9 |

