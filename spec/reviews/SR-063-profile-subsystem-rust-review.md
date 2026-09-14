---
id: SR-063
title: "Rust review — profile-separated rewrite architecture"
type: SpecReview
analysis: code-review
scope: "src/engine/, src/report.rs, src/replay.rs, src/equivalence.rs, tests/tc_053_profile_subsystems.rs"
review_set: subset
---

# Rust review — profile-separated rewrite architecture

## Summary

Applied the Rust review checklist to ownership, exhaustive matching, error
discriminants, panic/unsafe surface, conversions, resource limits, canonical wire
admission, dependency resolution, and requirement-traced tests. Every discovered
Rust finding was fixed and the complete implementation gate set passes.

## Verdict

**PASS AFTER REMEDIATION.** The production path contains no unsafe code, unchecked
wire/persistence conversion, unbounded traversal, async/lock hazard, stub, test-only
behavior branch, or recoverable caller-input panic. No finding remains open.

## Assurance Context

AP-001 (`spec/assurance/AP-001.md`) governs the reviewed rewrite candidate. The
explicit comparison baseline is `c26b21ca90175d3e4bed6ac23a0401d3f24c7947`, and
the evaluated implementation revision is
`62a76c50c3f93352db4088ebac4a771ee2ef199f`.
The review exercised the semantic-drift, false-completion, and context-substitution
impacts through real tl-syntax, tl-mltl, and Quire Observation owner types.

No refreshed Quoin assurance chain, independence assessment, or human release
decision was available because those qualification activities are excluded here.
Architecture and selected producer-reliance context came from the reviewed
PLAN-010/ADR-003 set and exact immutable dependency pins. Active exceptions: none.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-6301 | high | **FIXED:** three internal node lookups used direct indexing and `as usize`; all graph lookups now use checked conversion and bounds-aware access, preserving typed non-success instead of a panic. | `src/engine/mod.rs` |
| FND-6302 | medium | **FIXED:** one past `EvaluatorError` variant collapsed original and rewritten owner failures and discarded a successful first result; distinct typed variants now preserve the refusal locus and admitted side. | `src/equivalence.rs:95`; `src/equivalence.rs:618` |
| FND-6303 | high | **FIXED:** caller-controlled report/replay JSON could allocate or recurse without owner ceilings; input bytes are capped before parsing, traversal is iterative, output serialization is bounded, and exact canonical bytes are required. | `src/report.rs` |
| FND-6304 | medium | **FIXED:** count and node identity conversions at report/resource boundaries used implicit or lossy forms; checked conversions now conservatively refuse overflow. | `src/engine/mod.rs`; `src/equivalence.rs` |
| FND-6305 | medium | **FIXED:** profile modules initially delegated all policy to one shared matcher and therefore did not state ownership in their code shape; exhaustive temporal matches now live under the owning module. | `src/engine/future.rs`; `src/engine/past.rs` |

## Rust checklist result

- Public types and functions have rustdoc; warning-denied rustdoc passes.
- External payloads use closed serde shapes and canonical bounded readers.
- Test code uses real owner APIs with no mock of the behavior under test.
- TC-053 carries exact test-case and acceptance-criterion traces and asserts values,
  refusal codes, identities, revisions, bounds, and wrong-operator counterexamples.
- The sole production `expect` is confined to the crate-private JSON hash helper;
  every call site is a closed derive-based JSON value with string map keys, and the
  invariant is documented at the call boundary.
- `cargo fmt --check`, strict Clippy, 65 implementation tests, Cargo deny, exact
  MSRV, release, rustdoc, doctest, provenance, and unsafe gates pass.
- No qualification or assurance producer result is claimed.
