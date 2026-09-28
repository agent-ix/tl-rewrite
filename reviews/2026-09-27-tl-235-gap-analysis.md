---
id: SR-093
title: "Gap analysis of TL-235 and TL-231 rewrite qualification port"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/tl-rewrite@7e9d091beb0216770d5150b4f49215500caf1426; TL-235, TL-231, FR-019-AC-3, TC-076, Cargo.toml, Cargo.lock, benches/README.md, benches/input-digests.json, benches/rewrite_rules.rs, tests/infinite_owner_corpus.rs, tests/shared_assurance.rs, spec/requirements/FR-019-infinite-rule-applicability.md, spec/infinite-rewrite-test-matrix.md, spec/test-matrix.md"
review_set: subset
---

## Summary

Reviewed the focused rewrite adjunct against TL-235, TL-231, FR-019-AC-3, and TC-076, including matrix-to-test alignment. This is not a claim that the cross-crate tickets or Campaign are complete.

## Verdict

CONDITIONAL. The TC-076 matrix row is syntactically backed, but its claimed preservation of all four owner refusals is not semantically demonstrated.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-076 claims four preserved owner refusals, but two cases never call a parser, trace admission, rewriter, or evaluator refusal path. | spec/infinite-rewrite-test-matrix.md:40; tests/infinite_owner_corpus.rs:116-125 |

## Coverage

Quire coverage reports 166/166 backed rows, no unbacked rows or status lies, including the TC-076 tracking tag. The finite-prefix subject reaches `evaluate_prefix_safety`. TL-235 paired results and TL-231 mutation rates are intentionally pending and are not marked complete by this adjunct. Semantic alignment was evaluated here under the dispatching review brief; no plan-completion claim is made.

## Dispositions

Round 1 reviewed `45011e2c6e5fb1b16f0c99cb7a6c70cd064dc912`. Both formerly raw-field-only negatives now require typed owner admission refusals. The TC-076 focused integration test passes.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 45011e2c6e5fb1b16f0c99cb7a6c70cd064dc912 |
