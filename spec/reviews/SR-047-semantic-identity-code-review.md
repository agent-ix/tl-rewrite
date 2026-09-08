---
id: SR-047
title: "Rust code review — semantic rewrite identity"
type: SpecReview
analysis: code-review
scope: "src/rewrite.rs, src/equivalence.rs, tests/rewrite.rs, Cargo.toml, Cargo.lock, spec/requirements/FR-002-bounded-engine.md, spec/test-matrix.md"
review_set: base
---

# Rust code review — semantic rewrite identity

## Summary

Reviewed the semantic-identity amendment from the public `FormulaDocument`
boundary through interning, compaction, rewrite report bindings, replay-step
fingerprints, cycle fingerprints, and equivalence reports. The implementation
uses `NodeKind` as the semantic key while preserving the first encountered
`SourceSpan` only as diagnostic provenance. It does not add or alter any local
assurance runner, Python helper, Make target, or hosted-CI trigger.

The review found one medium defect before release: `after_sha256` serialized a
full `Node`, allowing a source span to alter a replay-step and rolling digest.
It now serializes `NodeKind`, matching the semantic identity used by the rest of
the engine. TC-040 covers both `p1 | p1` and `p1 -> p1` with distinct parser
offsets: input, request, output, and step semantic digests agree, while the
retained diagnostic step span is proved distinct.

Local Rust gates passed before rebase: formatting; targeted rewrite, replay,
and equivalence tests; strict workspace Clippy; and Cargo Deny (advisory
existing allow-list and exact-git wildcard warnings only). Human review and the
specification workflow's final acceptance remain required.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1601 | medium | FIXED before review request: replay-step `after_sha256` included diagnostic span provenance, changing the rolling fingerprint for semantically identical input. Digest `NodeKind` only; retain the span solely in `RewriteStep::source_span`. | FR-002-AC-4, TC-040 |
| FND-1602 | low | No unsafe code, panic-prone production indexing, nondeterministic container, or new assurance execution path was introduced by this amendment. | FR-002, NFR-001, FR-006 |
