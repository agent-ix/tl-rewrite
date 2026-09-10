---
id: SR-041
title: "Risk and complexity review of the hosted ix-flow package-specification domain"
type: SpecReview
analysis: risk-complexity
scope: "NFR-003-AC-7 through NFR-003-AC-12 and TC-039 for tl-rewrite issue 33"
review_set: all
---

## Summary

Technical risk is high and volatility is medium: npm package-spec syntax and
shell/YAML spellings are external contracts, while a false negative can admit a
different executable than the reviewed release. The mitigation is a bounded
fail-closed Rust parser with table-driven mutations and a benign-metadata
positive control.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-4101 | low | The external syntax surface is volatile but bounded by AC-8 and AC-12: named alternate families and any unclassifiable argument are refused rather than guessed. | NFR-003-AC-8, NFR-003-AC-12, TC-039 |
| FND-4102 | high | **FIXED after exact-head review:** crossing YAML and shell grammars created multiple fail-open paths. The mitigation now orders YAML run extraction before shell tokenization and separately covers quoted keys, word-boundary comments, word-internal hashes, and npm aliases. | NFR-003-AC-7, NFR-003-AC-9, TC-039, tl-rewrite#34 review |

## Risk Register

| Req | Technical Risk | Volatility | Driver | Mitigation |
| --- | --- | --- | --- | --- |
| NFR-003-AC-7/8/12 | High | Medium | npm specs plus YAML/shell token boundaries | Ordered YAML/shell classifiers and per-family mutation table |
| NFR-003-AC-9 | Low | Low | Metadata false positive | Step-name and comment positive controls |
| NFR-003-AC-10/11 | Low | Medium | Workflow/runtime version drift | Exact trigger set and executable version observation |

The top hazard is a false-negative alternate install. No concurrency,
distributed coordination, cryptographic primitive, hard performance bound, or
native-language contract is introduced.
