---
id: SR-086
title: "TL-246 V9 rewrite baseline gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/tl-rewrite@ac48980c03c7f87a7311e9536ac1e975fb13e1b4; benches/rewrite_rules.rs, benches/input-digests.json, benches/README.md, Cargo.toml, Cargo.lock; compared with origin/main at e48cf0b6b782d0d40aa3b56708a43558287f4ede and parent 9056b8b31f3f7f3772d2bd811c4f6edd2d89b509"
review_set: subset
---

## Summary

Ticket: TL-246. Checked whether this baseline staging change supplies the V9 harness inputs and result checks expected by the generated Campaign, without treating its presence as a measured paired distribution.

## Verdict

**PASS** — No implementation gap in this scoped baseline staging change. The Campaign still requires measured paired runs, complete collections, and independent replay.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage observations

- The three case names, application counts, canonical input digests, throughput units, Criterion sample count, warmup, and measurement minimum match current main exactly.
- The baseline's own locked source graph compiles, and all three benchmark smoke cases verify the public rewrite outcome.
- No timing result or accepted Campaign collection was produced by this review. A baseline branch and a successful smoke test alone do not meet TL-246 acceptance.
