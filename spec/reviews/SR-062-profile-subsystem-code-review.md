---
id: SR-062
title: "Code review — profile-separated rewrite architecture"
type: SpecReview
analysis: code-review
scope: "PLAN-010 Task-009 tl-rewrite allocation; FR-010; src/, tests/tc_053_profile_subsystems.rs"
review_set: subset
---

# Code review — profile-separated rewrite architecture

## Summary

Reviewed the complete tl-rewrite Task-009 allocation against FR-010, ADR-003,
TC-053, the established public API, and the exact promoted TL owner revisions.
All architecture, owner-boundary, canonical-reader, attribution, and provenance
defects found during review were repaired in the candidate.

## Verdict

**PASS AFTER REMEDIATION.** Profile-specific temporal policy now belongs to the
selected profile subsystem, shared Boolean algebra has one private implementation,
report/replay own their wire behavior, and every successful graph and past
comparison crosses the real pinned owner APIs. No finding remains open.

## Assurance Context

AP-001 (`spec/assurance/AP-001.md`) applies because the change affects one exact
source candidate, catalog/dependency set, semantic-drift boundary, bounded-success
classification, and context-bound replay. The comparison baseline is
`c26b21ca90175d3e4bed6ac23a0401d3f24c7947`; the reviewed implementation revision is
`62a76c50c3f93352db4088ebac4a771ee2ef199f`. Available context included MRS-001,
FR-001 through FR-010, TM-001, AP-001, the nine-repository PLAN-010/ADR-003 review
set, current source/tests, Cargo resolution, Quire 0.32.0 coverage, and the
non-qualification implementation gates.

Retained assurance outputs were deliberately not regenerated because qualification
work is outside this implementation allocation. Human release acceptance is not
available and is not claimed. The installed Quire module also has contradictory
status-column contracts (`Coverage Status` for structural validation, `Status` for
status-lie classification), so status-lie classification is unavailable even though
row binding is complete. Selected independent assurance/attestation evidence is
therefore stale or unavailable for this candidate; it does not substitute for the
executable owner-boundary tests. Active exceptions: none.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-6201 | high | **FIXED:** the first split left both temporal rule bodies in the shared engine behind a domain flag; future and past now own their exact temporal matches, with only profile-independent Boolean algebra in a private helper. | `src/engine/future.rs:16`; `src/engine/past.rs:16`; `src/engine/boolean.rs:20` |
| FND-6202 | medium | **FIXED:** report and replay initially remained façade modules over types and decoding owned by the engine; each subsystem now owns its complete public wire type/reader or replay behavior. | `src/report.rs`; `src/replay.rs`; `src/engine/mod.rs:3` |
| FND-6203 | high | **FIXED:** successful rebuilt graphs previously relied on constructor validation only; every candidate is now canonically serialized and re-admitted through the real pinned tl-syntax strict reader before output can escape. | `src/engine/mod.rs:446`; FR-010-AC-2 |
| FND-6204 | high | **FIXED:** public report/replay deserialization had no caller-lowerable byte, depth, or string ceilings and admitted noncanonical encodings; bounded canonical readers and report re-execution now fail closed. | `src/report.rs:560`; `src/report.rs:568`; `src/replay.rs:172`; FR-010-AC-5 |
| FND-6205 | high | **FIXED:** past equivalence existed only in legacy/test evaluator paths; the production API now invokes tl-mltl's temporal result producer and strict reader, preserves non-values, and makes original/rewrite refusal loci distinct. | `src/equivalence.rs:580`; FR-010-AC-4 |
| FND-6206 | medium | **FIXED:** exact dependency-version fields caused the provenance gate's field-order-sensitive manifest regex to reject valid revision pins; the parser now reads inline dependency fields independently of order and refuses absent/duplicate identity fields. | `scripts/check_provenance.py:214`; `Cargo.toml` |
| FND-6207 | medium | **FIXED:** FR-010 originally required both byte-identical reports and mandatory dependency-revision advancement; AC-3 now preserves all unaffected bytes while requiring revision fields and their derived digests to advance truthfully. | `spec/requirements/FR-010-organize-profile-rewrite-subsystems.md`; `tests/wire_records.rs` |

## Gates

- Complete explicit non-qualification suite: 65 passed, 0 failed, 0 ignored.
- TC-053: 3 passed, covering all five FR-010 criteria.
- Quire 0.32.0: FR-010 5/5 and repository 125/125 rows backed.
- `cargo fmt --check`, strict all-target/all-feature Clippy, all-target check,
  release build, Rust 1.98.1 check, warning-denied rustdoc, doc tests,
  `cargo deny check`, unsafe audit, and corpus/provenance checks passed.
- Qualification, assurance-input, attestation, and release-decision lanes were not run.
