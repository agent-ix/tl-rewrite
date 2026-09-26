---
id: SR-084
title: TL-250 infinite rewrite code review
type: SpecReview
analysis: code-review
scope: "agent-ix/tl-rewrite@5cb3fbc981d99d1d24aaf69e86f5cd0ddf79a65c; Cargo.toml, Cargo.lock, deny.toml, fuzz/Cargo.toml, fuzz/fuzz_targets/infinite_rewrite.rs, src/catalog.rs, src/disposition.rs, src/infinite.rs, src/lib.rs, src/report.rs, src/engine/boolean.rs, tests/future_lowering_parity.rs, tests/infinite_conformance.rs, tests/infinite_disposition.rs, tests/infinite_fuzz_seeds.rs, tests/infinite_rules.rs, tests/oracle_finite_rewrite.rs, tests/tc_053_profile_subsystems.rs, tests/wire_records.rs"
review_set: subset
---

## Summary

Ticket: TL-250. Reviewed the frozen infinite-profile feature implementation and Rust tests against FR-019, FR-020, FR-021, NFR-005, and FR-010. The new engine duplicates the existing Boolean rule authority, and its new oracle dependency fails the repository's dependency-source gate.

## Assurance Context

AP-001 (`spec/assurance/AP-001.md`) applies to the exact source, catalog, dependencies and conformance corpus. The reviewed source is 5cb3fbc981d99d1d24aaf69e86f5cd0ddf79a65c against origin/main 8330a9fc27c5424bc58b6c440254dfba28aac192. Checked semantic-drift and false-completion scenarios using the catalog, oracle paths, replay logic, tests, and available local gates. CAC-001 and MP-001 exist in the repo; no completed measurement or producer-reliance decision for this candidate was available. No exception was recorded. Qualification and release assurance are halted by team direction, so no assurance campaign was run.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Infinite rewrite duplicates the existing Boolean rule implementation; `bool.*` behavior now has two code owners and can drift. | src/infinite.rs:167; src/engine/boolean.rs:7; FR-010-AC-1 |
| FND-002 | high | New dev-only `tl-oracle` git dependency is absent from `deny.toml` source allow-list, so `cargo deny check` fails on this candidate. | Cargo.toml:28; deny.toml:31; FR-020-AC-2 |
| FND-003 | medium | The fuzz target silently skips an admitted input when the before-oracle returns `ResourceIncomplete` or `NoPeriodicFixedPoint`, so skipped applicable cases pass. | fuzz/fuzz_targets/infinite_rewrite.rs:100; FR-020-AC-3 |

## Verdict

FAIL. FND-001 and FND-002 block merge; FND-003 leaves the fuzz evidence incomplete. The generated rule fixture test and independent oracle tests exercise real source paths, but they do not cure the skipped fuzz cases.

## Gates

`cargo fmt --check`: pass. `cargo clippy --workspace --all-targets --all-features -- -D warnings`: pass. Five focused integration targets: 27 tests passed. `cargo deny check`: fail, source-not-allowed for `tl-oracle`. No CI workflow change. The direct all-target suite's `shared_assurance` failure was reported by the coder; guarded-ci was excluded because it invokes halted assurance. Git dependency revisions for tl-syntax 9a4316e, tl-parse 4dc67db, and tl-mltl 6798fbd are not on their respective origin/main branches and must be repinned after producer feature merges.

## Dispositions

Round 1 reviewed `dec8a70f12609cd74eedfb9dd7a9cee032c0fcdd`. Each original finding was checked against the fix commit and the required local gate.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | dec8a70f12609cd74eedfb9dd7a9cee032c0fcdd |
| FND-002 | fixed | dec8a70f12609cd74eedfb9dd7a9cee032c0fcdd |
| FND-003 | fixed | dec8a70f12609cd74eedfb9dd7a9cee032c0fcdd |
