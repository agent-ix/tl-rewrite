---
id: SR-047
title: "Rust code review — semantic rewrite identity"
type: SpecReview
analysis: code-review
scope: "src/rewrite.rs, src/equivalence.rs, tests/rewrite.rs, tests/wire_records.rs, Cargo.toml, Cargo.lock, deny.toml, spec/requirements/FR-002-bounded-engine.md, spec/requirements/FR-007-context-bound-reports.md, spec/test-matrix.md"
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

Rebase review then caught the stale TC-035 v1 snapshot. The changed digest was
the required effect of removing spans from formula-level identity, but it
contradicted FR-007's earlier byte-compatibility wording. SR-046 records the
specification correction; TC-035 now pins the corrected pre-release v0.1 bytes
instead of silently updating a golden under the old promise.

Two mutations falsified the closing claims after the correction. Restoring
`(NodeKind, Option<SourceSpan>)` as the pass and compaction interner key made
TC-040 fail because the idempotence/reflexivity step disappeared. Restoring the
full `FormulaDocument` as the v1 input/request digest source made the real
parser-seam TC-040 fail with distinct identities for `p1|p1` and
`( p1 ) | p1`. Both mutations were removed before the final gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1601 | medium | FIXED before review request: replay-step `after_sha256` included diagnostic span provenance, changing the rolling fingerprint for semantically identical input. Digest `NodeKind` only; retain the span solely in `RewriteStep::source_span`. | FR-002-AC-4, TC-040 |
| FND-1602 | high | FIXED during rebase review: the semantic digest correction changed all three TC-035 v1 snapshots while FR-007 still claimed the prior exact bytes. The implementation keeps the v1 schemas and fields; FR-007 now authorizes and TC-035 pins the one pre-release v0.1 correction. | FR-002-AC-4, FR-007-AC-5, TC-035, TC-040 |
| FND-1603 | medium | FIXED during full-gate review: the new dev-only parser seam dependency was absent from Cargo Deny's closed Git-source allow-list, so `make ci` failed at `cargo deny check sources`. `deny.toml` now names the exact repository class and documents the dev-only boundary; the dependency revision is reachable from tl-parse main. | FR-002-AC-4, TC-040, Cargo.toml, Cargo.lock, deny.toml |
| FND-1604 | low | No unsafe code, panic-prone production indexing, nondeterministic container, or new assurance execution path was introduced by this amendment. | FR-002, NFR-001, FR-006 |
