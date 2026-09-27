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
| FND-004 | still-open | A compatible landed tl-oracle revision does not exist yet; old tl-syntax 9a4316e and landed 6aa9b11 have distinct Rust types, so oracle-backed tests fail E0308. |
| FND-004 | fixed | 724e86e448456785610013ce899419f5bbf26b3c |

Round 2 reviewed `d48d1c32f4b52f5ff6e8c2c1adb72203f455872f`; a new finding follows.

## Review round 2 — landed producer repin

Reviewed `d48d1c32f4b52f5ff6e8c2c1adb72203f455872f` against `origin/main` (`8330a9f`), including the seven-file `924ea61..d48d1c3` producer-repin diff, `Cargo.toml`, `Cargo.lock`, `fuzz/Cargo.toml`, `fuzz/Cargo.lock`, `src/lib.rs`, `tests/tc_053_profile_subsystems.rs`, `tests/infinite_rules.rs`, and the unchanged feature implementation and oracle fuzz seam. Original FND-001 through FND-003 remain fixed. No CI workflow change, new vendoring, source-level duplicate, or newly exposed production panic path was found in the repin. AP-001 remains applicable; no measurement, producer-reliance acceptance, or exception is claimed for this revision.

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | high | The dev-only oracle is pinned to the old tl-syntax source, so the newly repinned infinite oracle suite cannot compile and independent rule soundness cannot be checked. | Cargo.toml:25,28; tests/infinite_rules.rs:417; FR-020-AC-1 |

`tl-oracle` at `6a2bec0` consumes `tl-syntax` `9a4316e`, whereas this candidate uses landed `6aa9b11`; Rust treats their `InfiniteFormulaDocument`, `LassoTraceDocument`, and `NodeId` as different types. `cargo test --offline --test infinite_rules --no-default-features --features infinite-trace --no-run` fails with eight E0308 errors. The same identity split reaches the oracle-driven fuzz target. A compatible independently owned oracle revision is required before this test dependency can be repinned and the acceptance checks rerun; there is no compatible landed oracle at this review.

## Round 2 verdict and checks

**FAIL** — FND-004 blocks merge. `cargo fmt --check`, `cargo deny check sources --disable-fetch`, targeted Quire validation, and `git diff --check` passed; 65 focused non-oracle tests were reported by the coder. The oracle integration compile failed as described; no oracle verdict or fuzz soundness result exists for this head. Aggregate gates and qualification work were outside Peter's feature-only direction.

## Disposition round 3 — oracle pin and type identity

Reviewed `724e86e448456785610013ce899419f5bbf26b3c` against the original base and `d48d1c3`. FND-004 is fixed by this commit: `Cargo.toml` and both locks pin merged tl-oracle `2391e5b7402ae02e8d138f3901b43a1f07acd1b6`, which consumes landed tl-syntax `6aa9b11`. `cargo tree --offline -i tl-syntax` shows one source shared by tl-mltl, tl-parse, tl-oracle and tl-rewrite. The previously failing `infinite_rules` suite now executes 16/16, including TC-070/071/072; `cargo check --offline --manifest-path fuzz/Cargo.toml --bin infinite_rewrite` compiles the real oracle fuzz target. `infinite_fuzz_seeds` 1/1, `wire_records` 3/3, provenance, deny sources, fmt and diff check pass. The changed wire bytes are exact-pin snapshot values; no production rule body changed. AP-001 remains applicable; no qualification or release-assurance conclusion is claimed.

**Round 3 verdict: PASS.** No open code-review findings at this head.
