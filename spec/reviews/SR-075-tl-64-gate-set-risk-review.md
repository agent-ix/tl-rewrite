---
id: SR-075
title: "risk-complexity review of NFR-004 (TL-64 gate-set-integrity remediation)"
type: SpecReview
analysis: risk-complexity
scope: "spec/requirements/NFR-004-gate-set-integrity.md"
review_set: all
---

## Summary

Assessed implementation risk and volatility now that the requirement has
been broadened (SR-074) to cover the full execution-control class plus
environment control and the documented-path check. The requirement is wider
than a first read suggests; flagging the two riskiest sub-mechanisms for the
implementation plan to sequence carefully rather than treat as one
undifferentiated task.

## Findings

| ID      | Severity | Summary                          | Refs   |
| ------- | -------- | --------------------------------- | ------ |
| FND-001 | medium   | The completion-record mechanism (AC-3/AC-4/AC-5) requires every one of the 13 `ci` prerequisite recipes to be touched (each must write its own record on success) — the widest-blast-radius piece of this requirement, since it edits recipes owned by other requirements (FR-001 through FR-010, NFR-001 through NFR-003) even though this NFR does not own their correctness. Recommend `spec-to-plan` sequence this as its own task with a mechanical, low-risk pattern (e.g. a shared Make function each recipe calls) rather than 13 hand-written edits, to keep the change reviewable and to avoid accidentally altering a gate's actual behavior while only intending to add a record. | NFR-004 |
| FND-002 | medium   | `$(eval` and `include` handling (AC-1) is a textual ban rather than a semantic Make parse, which is deliberately conservative (matching the removed guard's own approach per the Makefile header) but means a legitimate future use of either construct — however benign — trips the refusal. This is an accepted false-positive-over-false-negative trade-off given the requirement's purpose (fail closed on anything unanalyzable), not a defect; recorded so a future contributor who hits the refusal understands it is intentional rather than a bug to route around. | NFR-004 |
| FND-003 | low      | AC-2's `MAKEFLAGS`/invocation-environment control is the least precedented mechanism here (no existing repository code does anything like it) and the most likely to need iteration once actually implemented against real `make` behavior across platforms. Low severity because it is additive safety, not a correctness dependency for the other ACs — its own tests can fail without invalidating AC-1/AC-3/AC-4/AC-5. | NFR-004 |
