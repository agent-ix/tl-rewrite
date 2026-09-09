---
id: SR-018
title: "Property-grounding review — rewrite equivalence baseline"
type: SpecReview
analysis: base
scope: "FR-001, FR-004, NFR-001, TM-001, tests/property.rs"
review_set: base
---

# Property-grounding review — rewrite equivalence baseline

## Summary

Quire's property export identifies fifteen extractable obligations. This review
maps each to its existing deterministic evidence or to TC-037, which grounds
the finite temporal and reflexive Boolean rewrite families against the exact
bounded evaluator. It preserves the declared evaluator and domain ceilings;
random generation does not claim universal proof beyond those bounds.

Quoin `advise` could not run because its version probe rejects installed Quire
0.31.0 despite the tool satisfying the documented minimum. That is an upstream
tooling finding, not a local adapter or replacement opportunity.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1801 | low | TC-037 grounds the reusable finite rewrite-family property. The other extractable criteria are already enforced by their named deterministic matrix controls; no duplicate generator is warranted. | FR-001-AC-2, FR-004-AC-1, TM-001 |
| FND-1802 | medium | DEFERRED: Quoin `advise` cannot read the installed Quire 0.31.0 JSON contract because its version detection fails. No local substitute is introduced. | quoin advise, quire-cli |
| FND-1803 | low | Fuzz-target decision remains deferred until the property baseline is measured; mutation testing is explicitly out of scope for this baseline. | tl-rewrite#27 |
