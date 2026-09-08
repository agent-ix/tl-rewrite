---
id: SR-020
title: "Risk and complexity review of the complete tl-rewrite specification corpus"
type: SpecReview
analysis: risk-complexity
scope: "spec/**/*.md"
review_set: all
---

# SR-020: Risk and complexity review of the complete tl-rewrite specification corpus

## Summary

The corpus names material semantic-drift, false-completion, and
context-substitution scenarios in AP-001, with bounded testing and review as
mitigations. It does not preserve the per-requirement technical-risk and
volatility register this analysis requires, so a future plan cannot recover the
reviewed priority order from the specification alone.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2001 | medium | No risk register scores every FR/StR/NFR on technical risk and volatility or names mitigations for each high axis; AP-001 identifies material scenarios but is not the per-requirement register required for task sequencing. | AP-001, FR-001 through FR-007, NFR-001 through NFR-003 |
