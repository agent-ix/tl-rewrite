---
id: SR-085
title: TL-250 infinite rewrite feature gap analysis
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/tl-rewrite@5cb3fbc981d99d1d24aaf69e86f5cd0ddf79a65c; spec/requirements/FR-010-organize-profile-rewrite-subsystems.md, spec/requirements/FR-019-infinite-rule-applicability.md, spec/requirements/FR-020-independent-rewrite-soundness.md, spec/requirements/FR-021-infinite-rewrite-refusals.md, spec/requirements/NFR-005-infinite-rewrite-identity-and-bounds.md, spec/infinite-rewrite-test-matrix.md, spec/test-matrix.md, src/catalog.rs, src/disposition.rs, src/infinite.rs, fuzz/fuzz_targets/infinite_rewrite.rs, tests/infinite_conformance.rs, tests/infinite_disposition.rs, tests/infinite_fuzz_seeds.rs, tests/infinite_rules.rs, tests/oracle_finite_rewrite.rs"
review_set: subset
---

## Summary

Ticket: TL-250. Checked FR-019/020/021 and NFR-005 acceptance criteria against production code, the matrix, and tagged tests. Matrix binding is complete, but TC-072's fuzz path can pass without independent comparison for admitted cases.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | TC-072 does not enforce FR-020-AC-3's refusal of skipped applicable cases: resource-incomplete and non-periodic before-oracle results leave the fuzz iteration green. | fuzz/fuzz_targets/infinite_rewrite.rs:100; FR-020-AC-3; TC-072 |

## Verdict

FAIL. `quire coverage --scope . --json` reports 164/164 backed matrix rows, no unbacked rows, and no status lies. This is a semantic test-oracle gap despite the successful tag binding. The repo has no TL-250-specific plan bundle to assess task status. Broader qualification is halted; this review is limited to the ticket's feature behavior and its acceptance tests.

## Dispositions

Round 1 reviewed `dec8a70f12609cd74eedfb9dd7a9cee032c0fcdd`. Each original finding was checked against the fix commit and the required local gate.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | dec8a70f12609cd74eedfb9dd7a9cee032c0fcdd |
| FND-002 | still-open | FR-020 oracle acceptance is unexecuted at this exact head because the test-only oracle dependency cannot consume landed syntax types. |
| FND-002 | fixed | 724e86e448456785610013ce899419f5bbf26b3c |

## Review round 2 — landed producer repin

Reviewed `d48d1c32f4b52f5ff6e8c2c1adb72203f455872f`: FR-019/020/021 and NFR-005 against `spec/infinite-rewrite-test-matrix.md`, the tagged `tests/infinite_rules.rs` and `fuzz/fuzz_targets/infinite_rewrite.rs` oracle paths, and producer dependency closure in `Cargo.toml`/`Cargo.lock`. Original FND-001 remains fixed. Quire still reports 164/164 matrix rows backed, zero unbacked rows and zero status lies; that structural binding does not execute Rust tests. No TL-250 plan bundle exists.

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | TC-070 and TC-072 still claim implemented FR-020 oracle evidence although the oracle-backed integration suite cannot compile after the landed syntax repin. | spec/infinite-rewrite-test-matrix.md:34,36; tests/infinite_rules.rs:417; FR-020-AC-1,FR-020-AC-3 |

`tl-oracle`'s old tl-syntax type identity prevents `evaluate_documents` from accepting the candidate's landed syntax documents (eight E0308 errors). Thus the tagged tests and fuzz source are present but cannot establish the independent before/after oracle verdict required for enabled infinite rules on this exact head.

## Round 2 verdict

**FAIL** — FND-002 is an executable acceptance gap. It shares the dependency cause of code-review SR-084 FND-004. A compatible landed oracle, exact repin, and successful oracle/fuzz checks must precede a merge verdict.

## Disposition round 3 — executable FR-020 evidence

Reviewed `724e86e448456785610013ce899419f5bbf26b3c` against FR-020-AC-1/3, TC-070/072, the oracle-backed `tests/infinite_rules.rs`, `tests/infinite_fuzz_seeds.rs`, and `fuzz/fuzz_targets/infinite_rewrite.rs`. FND-002 is fixed by this commit: the merged oracle pin resolves the same landed tl-syntax document types, TC-070's independent comparison and wrong-rule counterexamples execute within the 16/16 passing infinite-rules suite, TC-072's seeded rewrite-then-oracle sweep executes, and the real fuzz target compiles with that oracle. Quire reports 164/164 backed rows, no unbacked rows or status lies. This is focused feature evidence, not an aggregate gate or verification campaign.

**Round 3 verdict: PASS.** No open gap-analysis findings at this head.
