---
id: SR-059
title: "Code review — shared past/history rewrite replay"
type: SpecReview
analysis: code-review
scope: "corpus/past-history, tests/past_history_corpus.rs, Makefile"
review_set: subset
---

# Code review — shared past/history rewrite replay

## Summary

Reviewed the retained corpus, production rewrite reports, exact graph outcomes,
and replay sensitivity against Task-005.

## Verdict

**PASS after remediation.** Both approved past folds and the unchanged Since
boundary run through the production rewriter, preserve formula-v2/profile
identity, match exact rule IDs and graphs, and reproduce under the replay API.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-5901 | high | Text expectations now map to exact output node/interval or semantic-identity assertions. | `tests/past_history_corpus.rs` |
| FND-5902 | medium | Corpus and engine status spellings use an explicit closed mapping. | `tests/past_history_corpus.rs` |
| FND-5903 | medium | Rule-ID mutation now requires deterministic replay mismatch. | `corpus_digest_and_rewrite_expectation_mutations_are_detected` |
