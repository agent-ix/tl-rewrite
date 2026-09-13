---
id: SR-057
title: "Rust review — past-profile rewrite implementation"
type: SpecReview
analysis: code-review
scope: "src/catalog.rs, src/rewrite.rs, src/equivalence.rs, src/lib.rs, tests/past_rewrite.rs, tests/future_lowering_parity.rs, Cargo.toml, Cargo.lock, deny.toml"
review_set: all
---

## Summary

Applied the repository and `/rust-review` checks to exhaustiveness, ownership,
error surfaces, resource bounds, unsafe/panic exposure, wire conversions,
dependency identity, and test discrimination. The implementation is idiomatic
Rust 1.75 and every review finding is resolved.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-5701 | high | FIXED: new non-exhaustive upstream enums first broke NodeKind traversal/remap and the test-only SemanticProfile bridge. Production matches now explicitly cover O/H/Y/S/T; the future-only test bridge explicitly rejects the past profile. | FR-009-AC-2, TC-047, TC-044 |
| FND-5702 | medium | FIXED: formula reconstruction now matches `FormulaSchemaVersion` exhaustively and delegates all topology/depth/profile validation back to tl-syntax. IDs indexed during rewriting come only from a previously validated topological document or this pass's checked emitter. | FR-009-AC-2, TC-047, TC-050 |
| FND-5703 | medium | FIXED: past rule application performs no interval expansion or arithmetic; maximum `u32` Triggered and unchanged Once intervals are covered, and existing node/work/application/iteration limits still fail before a partial output. | FR-009-AC-2, FR-009-AC-5, TC-047, TC-050 |
| FND-5704 | medium | FIXED: exact git pins and public revision constants now agree with Cargo.lock, and the Rust 1.75 all-target check compiles the same formula type through tl-rewrite and tl-mltl. Historical renamed W/M test dependencies remain explicitly isolated. | FR-009-AC-6, TC-030 |
| FND-5705 | low | Verified: strict Clippy is clean; rustdoc with warnings denied is clean; release and MSRV builds pass; unsafe audit is empty; no newly added production unwrap, expect, panic, allow, blocking-in-async, lock, cast, lossy wire integer, or unbounded allocation exists. | NFR-001, NFR-002 |
