---
id: SR-050
title: "Rust review — bounded rewrite property baseline"
type: SpecReview
analysis: code-review
scope: "tests/property.rs, spec/test-matrix.md and assurance census pins at f672605 over origin/main 033a687"
review_set: subset
---

# Rust review — bounded rewrite property baseline

## Summary

Reviewed the issue-27 property increment after merging current main. The branch
adds one bounded check for the four reflexive Boolean rewrite families and
retains the existing bounded Until/Release property. The new property exercises
the public rewrite path and the independent exact evaluator, and its test/matrix
identity is repository-unique as TC-046. Its closed family enum is exhaustively
iterated and the asserted rule-identity census prevents accidental duplication.

## Verdict

**PASS** — the check exercises the correct public boundary and an independent
evaluator for every member of its four-family Boolean domain. The fuzz boundary
is a separate evidence-method finding in SR-049 and is not hidden by this
verdict.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-5005 | low | No remaining Rust review defect was found in the bounded property slice at `f672605`; every reflexive Boolean family is executed exactly once and checked through the public rewrite and independent equivalence boundaries. | `tests/property.rs`; TM-001 TC-046 |

## Remediation history

FND-5001 and FND-5003 were fixed during main integration and the initial full
gate. FND-5004 was fixed at `f672605` by replacing the randomized selector with
exhaustive enumeration and an asserted unique rule-identity census.

## Review

| Check | Result |
| --- | --- |
| Real boundary | Both properties call public `rewrite` and `check_equivalence`; no mock or local reimplementation substitutes for either production boundary. |
| Oracle independence | Rewrite rule selection/application and exact trace evaluation are separate implementations. The oracle evaluates the pre/post graphs rather than asserting only that the rewriter returned its own expected status. |
| Domain and bounds | Temporal kinds and interval endpoints are generated from explicit finite domains. The Boolean families are a closed four-variant enum and are exhaustively enumerated. |
| Mutation sensitivity | A missing family, duplicate rule identity, missing rule step, wrong normalized status or unequal trace semantics changes a direct assertion. |
| Panic surface | The interval constructor unwrap is over a strategy that constructs only `1 <= start <= end <= 3`; output unwrap follows an asserted `Normalized` status and remains test-only. Production panic/unsafe surface is unchanged. |
| Async, locks and resources | No async task, lock, I/O, unbounded generator, or production allocation path is added. Proptest disables persistence and uses the repository's fixed case count. |
| Trace integrity | The new property carries TC-046 and the three claimed criterion IDs. Current main's TC-037..TC-045 remain distinct and intact. |
| Repository idioms | Existing `tests/property.rs` fixtures, public types, proptest configuration, formatting and stable-MSRV dependency set are reused; no dependency or gate changed. |

## Gates

The complete local `make ci` gate passed at reviewed implementation head
`655eccb` with the released module set selected through
`IX_FILAMENT_MODULES_PATH`:

- formatting and all-target/all-feature Clippy with warnings denied;
- 68 Rust tests, including both bounded property tests;
- corpus provenance, rule conformance, counterexample replay and normalization;
- cargo-deny, unsafe-comment audit, Rust 1.75 all-target/all-feature check and
  warning-free rustdoc;
- 119/119 specification artifacts structurally valid and grammar-clean;
- Quire coverage 100/100 rows, with 68/68/68 Rust candidates/tagged/bound;
- released shared-tool pins and the complete Quoin assurance chain.

Cargo-deny emitted only the existing unmatched-license-allowance warnings.
Quire emitted the known duplicate-module/archetype diagnostics from overlapping
module discovery. No hosted workflow was dispatched.

## Boundary

This review clears the bounded Boolean family-completeness claim. It does not
claim fuzz execution, mutation effectiveness, universal temporal equivalence, production
monitor qualification or closure of the cross-repository TL effectiveness
epic.
