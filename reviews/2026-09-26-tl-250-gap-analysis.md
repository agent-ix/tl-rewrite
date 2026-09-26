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
