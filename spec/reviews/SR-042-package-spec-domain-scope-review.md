---
id: SR-042
title: "Scope-boundary review of the hosted ix-flow package-specification domain"
type: SpecReview
analysis: scope-boundary
scope: "NFR-003-AC-7 through NFR-003-AC-12 and TC-039 for tl-rewrite issue 33"
review_set: all
---

## Summary

tl-rewrite owns inspection of its tracked hosted workflow and the Rust test
that admits one exact package specification. npm owns package installation and
the released ix-flow executable owns its version output; neither external tool
is reimplemented or treated as a release authority.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-4201 | low | No boundary gap remains: the change is internal TL assurance infrastructure and does not define a Quire grammar, authored temporal profile, TL source language, evaluator semantics, or export mapping. | NFR-003, TC-039, quire-spec-design-handoff |
| FND-4202 | high | **FIXED after exact-head review:** the earlier boundary conflated YAML document comments with comments interpreted by the shell inside YAML run scalars. The allocation now gives YAML selection to the workflow parser and executable comment/argument classification to the shell scanner. | NFR-003-AC-7, NFR-003-AC-9, TC-039 |

## Responsibility Allocation

| Requirement | Owner | Class |
| --- | --- | --- |
| NFR-003-AC-7/8/9/10/12 | tl-rewrite shared-assurance test boundary | cross-cutting |
| NFR-003-AC-11 | released ix-flow executable, observed by tl-rewrite | external contract |

Native Quire remains the sole editable formal-clause language. tl-rewrite
remains internal Rust temporal rewrite infrastructure and this issue makes no
claim that tl-syntax is an alternate user-authored source.
