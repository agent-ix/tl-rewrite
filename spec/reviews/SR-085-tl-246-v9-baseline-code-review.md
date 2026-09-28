---
id: SR-085
title: "TL-246 V9 rewrite baseline code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/tl-rewrite@ac48980c03c7f87a7311e9536ac1e975fb13e1b4; benches/rewrite_rules.rs, benches/input-digests.json, benches/README.md, Cargo.toml, Cargo.lock; compared with origin/main at e48cf0b6b782d0d40aa3b56708a43558287f4ede and parent 9056b8b31f3f7f3772d2bd811c4f6edd2d89b509"
review_set: subset
---

## Summary

Ticket: TL-246. Reviewed the one-file historical baseline staging change, the current V9 benchmark harness, the input corpus digests, and both locked dependency graphs. The head is a direct child of the named historical parent, and its benchmark file is byte-identical to current main.

## Verdict

**PASS** — No code or Rust finding in the scoped V9 baseline staging change. This is not a performance verdict or Campaign acceptance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Review evidence

- Only `benches/rewrite_rules.rs` differs from the historical parent. No CI workflow, dependency declaration, or lockfile changed in this PR.
- The head and current-main harness blobs are identical. The input digest JSON is also identical on both refs. Baseline and current graphs have different pinned TL source revisions by design; each lock resolves one syntax revision.
- Preflight and postflight checks validate normalized status, exact step count, and a one-node output. The timed closure invokes `rewrite` with the same three documents, budgets, Criterion settings, and `black_box` consumption as current main, without timed assertions.
- In a temporary archive of the exact head: `cargo fmt --check`, strict all-target/all-feature Clippy, `cargo deny check`, `cargo test --locked --offline --test rewrite` (12 passed), and `cargo bench --locked --offline --bench rewrite_rules -- --test` (three cases passed).
- The full `cargo test --locked --offline` run fails in `shared_assurance` because the temporary archive lacks `make assurance-inputs` products and `.venv-assurance`. Those gates were not counted as passing; the failures do not exercise the changed benchmark.
