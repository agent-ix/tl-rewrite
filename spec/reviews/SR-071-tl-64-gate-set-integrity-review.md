---
id: SR-071
title: "integrity review of NFR-004 (TL-64 gate-set-integrity remediation)"
type: SpecReview
analysis: integrity
scope: "spec/requirements/NFR-004-gate-set-integrity.md"
review_set: all
---

## Summary

Checked NFR-004 for internal completeness, consistency, and atomicity: does
every mechanism the Verification section describes have a matching
Acceptance Criterion, does the Scope's execution-control list match what the
Statement/AC actually check, and is bundling static inspection with
declared/executed reconciliation in one requirement defensible rather than
an accidental merge of two unrelated controls. One completeness gap found
and closed; atomicity judged sound and recorded rather than left implicit.

## Findings

| ID      | Severity | Summary                          | Refs   |
| ------- | -------- | --------------------------------- | ------ |
| FND-001 | medium   | The Verification section described a third negative control — a completion record removed or backdated while its gate stays declared — with no corresponding Acceptance Criteria row; an AC table that omits a described control is a gap between what the requirement says it verifies and what it commits to. Closed by adding NFR-004-AC-5, which states the removed/backdated-record behavior as a testable criterion. | NFR-004 |
| FND-002 | low      | Considered splitting NFR-004 into two requirements (static Makefile inspection vs. declared/executed reconciliation) since they are independently testable. Kept as one requirement: both mechanisms serve the same single guarantee named in the title — binding declared to executed — and splitting them would let one half ship without the other while still claiming the guarantee's name. No change made; recorded as a considered alternative. | NFR-004 |
