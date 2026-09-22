---
id: SR-065
title: "Property-grounding review — rewrite equivalence baseline"
type: SpecReview
analysis: base
scope: "FR-001, FR-004, NFR-001, TM-001, tests/property.rs"
review_set: base
---

# Property-grounding review — rewrite equivalence baseline

## Summary

Quire's property export identifies fifteen extractable obligations. This review
maps each to its existing deterministic evidence or to TC-054, which exercises
the finite temporal and reflexive Boolean rewrite-family domains against the
exact bounded evaluator. It preserves the declared evaluator and domain
ceilings and deterministically enumerates every Boolean family member.

Quoin `advise` could not run because its version probe rejects installed Quire
0.31.0 despite the tool satisfying the documented minimum. That is an upstream
tooling finding, not a local adapter or replacement opportunity.

Renumbered from SR-048/TC-046 to SR-065/TC-054 when rebasing onto current
`main`, whose own unrelated work had since claimed both ids (`agent-ix/tl-rewrite#27`).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-6501 | low | TC-054 grounds the reusable finite rewrite-family property. The other extractable criteria are already enforced by their named deterministic matrix controls; no duplicate generator is warranted. | FR-001-AC-2, FR-004-AC-1, TM-001 |
| FND-6502 | medium | DEFERRED: Quoin `advise` cannot read the installed Quire 0.31.0 JSON contract because its version detection fails. No local substitute is introduced. | quoin advise, quire-cli |
| FND-6503 | low | **Resolved by SR-066:** the property baseline is measured. No Fuzz-kind obligation can currently be selected because none is authored and Quoin's advisor version probe fails; no target is guessed. Mutation testing remains explicitly out of scope for this baseline. | tl-rewrite#27, SR-066 |
| FND-6504 | medium | **Fixed at `f672605`:** TC-054 uses a closed four-variant enum, loops over every member, rejects duplicate rule identities, and asserts the complete exercised population. | `tests/property.rs`; TM-001 TC-054; tl-rewrite#27 |
