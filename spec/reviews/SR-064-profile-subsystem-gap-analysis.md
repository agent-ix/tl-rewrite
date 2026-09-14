---
id: SR-064
title: "Gap analysis — PLAN-010 Task-009 tl-rewrite allocation"
type: SpecReview
analysis: gap-analysis
scope: "PLAN-010 Task-009 tl-rewrite allocation; FR-010; spec/test-matrix.md; src/, tests/"
review_set: subset
---

# Gap analysis — PLAN-010 Task-009 tl-rewrite allocation

## Summary

Audited the tl-rewrite allocation from requirement through production behavior,
public API, TC-053, regression tests, dependency provenance, and promotion gates.
All allocation-local requirements and findings are implemented; PLAN-010 Task-009
correctly remains in progress until this fourth and final TL allocation is merged.
The audited implementation revision is
`62a76c50c3f93352db4088ebac4a771ee2ef199f`.

## Verdict

**PASS FOR THE TL-REWRITE ALLOCATION.** FR-010 is backed 5/5, every new public
behavior has an owning requirement, and no source/test stub or deferred allocation
item remains. This verdict does not close the larger PLAN-010 campaign.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-6401 | medium | **FIXED:** initial profile modules did not own the temporal rule bodies promised by the architecture; temporal policy is now physically and behaviorally separated. | FR-010-AC-1; `src/engine/future.rs`; `src/engine/past.rs` |
| FND-6402 | medium | **FIXED:** replay reader evidence originally covered only the exact and one-under byte limit; TC-053 now exercises byte, depth, string, trailing, duplicate, unknown, reordered, and tampered cases across reports and replay. | FR-010-AC-5; `tests/tc_053_profile_subsystems.rs:593` |
| FND-6403 | medium | **FIXED:** the past-conformance report named the evaluator revision but omitted the syntax owner revision for its formula pair; both exact promoted revisions are now retained and asserted. | FR-010-AC-4; `src/equivalence.rs`; `tests/tc_053_profile_subsystems.rs:829` |
| FND-6404 | low | Existing TC-056/FR-013 tags are intentionally external PLAN-010 corpus traces rather than local TM-001 rows; Quire reports them as four unchanged untracked-symbol entries, and SR-061 retains their allocation review. | `tests/past_history_corpus.rs`; `spec/reviews/SR-061-past-corpus-replay-gap-analysis.md` |

## Coverage

- Target: PLAN-010 Task-009, tl-rewrite allocation; local spec root `spec/`;
  matrix `spec/test-matrix.md` (TM-001); source/test roots `src/`, `tests/`.
- Reconciliation: Quire 0.32.0 / engine `a874fb641cb70da83c8c8b23f9fea0a44255b88a`.
- Repository rows backed by tagged tests: 125/125; FR-010 criteria: 5/5;
  TM-001 cases: 51/51; Rust evidence symbols bound: 83/83.
- The installed traceability model cannot classify functional status lies because
  it expects `Status` while the same installed archetype structurally requires
  `Coverage Status`; backing counts remain available, status classification does not.
- New public behavior groups inventoried: profile dispatch, shared Boolean algebra,
  graph owner re-admission, report admission/re-execution, replay admission,
  owner-backed past comparison, typed comparison failures, and dependency attribution.
  Untraced new behavior: 0. Source stubs: 0. Test stubs: 0.
- Complete explicit non-qualification execution: 65/65 tests passed.
- Optional semantic gap-analysis step: skipped because it was not separately opted
  into; code review still checked FR-010 intent against each TC-053 assertion and
  production path.
- Central PLAN-010 Task-009 promotion: 3/4 TL repositories merged; this allocation
  is the remaining fourth repository and is not counted merged by this artifact.
