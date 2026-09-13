---
id: SR-056
title: "Code review — past-profile rewrite implementation"
type: SpecReview
analysis: code-review
scope: "Cargo.toml, Cargo.lock, src/catalog.rs, src/rewrite.rs, src/equivalence.rs, src/lib.rs, tests/past_rewrite.rs, tests/future_lowering_parity.rs, tests/wire_records.rs, README.md, docs/DER-001-rule-derivations.md, FR-007, FR-009, TM-001"
review_set: all
---

## Summary

Reviewed the complete change from profile selection through catalog priority,
graph traversal/rebuild, rule application, report/replay identity, conformance
refusal, dependency provenance, and tests. All findings were fixed; no mock,
stub, tautological assertion, qualification execution, or partial-output path
remains in the task scope.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-5601 | high | FIXED: the first implementation rebuilt every graph with `FormulaDocument::new`, which would down-convert formula-v2 and reject valid past nodes. One exhaustive schema-preserving constructor now serves pass rebuild and compaction, with TC-047 covering every past operator. | FR-009-AC-2, TC-047 |
| FND-5602 | high | FIXED: representative Boolean coverage was insufficient for a catalog that admits all B01–B24. TC-048 now derives every Boolean fixture from the checksum-bound corpus, exercises its exact rule, and compares both clocks over all Boolean valuations. | FR-009-AC-3, TC-048 |
| FND-5603 | medium | FIXED: the future-lowering source census initially treated the newly canonical `StrongPrevious` name and all new past NodeKind variants as forbidden derived-future operators. The scanner now distinguishes strong Release spellings and enumerates all canonical past primitives, while retaining its synthetic controls. | FR-008-AC-4, TC-044, FR-009-AC-2 |
| FND-5604 | high | FIXED: dependency pins truthfully changed v1 report bytes while FR-007 and TC-035 froze the prior dependency identity. The reviewed compatibility rule and candidate snapshots now preserve schemas/statuses and exact same-dependency determinism without lying about compiled revisions. | FR-007-AC-5, FR-009-AC-6, TC-035 |
| FND-5605 | medium | FIXED: initial tests did not exercise every O/H/Y/S/T operand traversal, maximum interval identity, report wire round-trip, or future-only conformance refusal. TC-047, TC-049, TC-050, and TC-051 now cover each boundary. | FR-009-AC-2, FR-009-AC-4, FR-009-AC-5 |
| FND-5606 | low | Verified: production code contains no new panic/unwrap, unsafe block, blocking operation, nondeterministic collection, unchecked integer conversion, extension callback, foreign qualification dependency, or parser/monitor implementation. Test unwraps construct invariant fixtures only. | NFR-001, NFR-002, FR-009 |
