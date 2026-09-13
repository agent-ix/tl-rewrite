---
id: SR-050
title: "Rust review — bounded rewrite property baseline"
type: SpecReview
analysis: code-review
scope: "tests/property.rs, spec/test-matrix.md and assurance census pins at 655eccb over origin/main 033a687"
review_set: subset
---

# Rust review — bounded rewrite property baseline

## Summary

Reviewed the issue-27 property increment after merging current main. The branch
adds one bounded property for the four reflexive Boolean rewrite families and
retains the existing bounded Until/Release property. The new property exercises
the public rewrite path and the independent exact evaluator, and its test/matrix
identity is repository-unique as TC-046.

## Verdict

**PASS** — no unresolved Rust finding in the property increment. The fuzz
boundary is a separate evidence-method finding in SR-049 and is not hidden by
this verdict.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-5001 | medium | **Fixed during main integration:** the old branch's TC-037 and SR-018 identities collided with current main. The same property/review bodies now use unique TC-046 and SR-048 identities, retaining main's TC-037..TC-045 and SR-018..SR-047 unchanged. | TM-001; SR-048 |
| FND-5003 | medium | **Fixed during the full local gate:** adding SR-048 through SR-050 changed the fail-closed tracked-source census from 165 to 168, and TC-046 changes the released-module coverage pin from 99 to 100. Both assertions now state their exact arithmetic rather than weakening the ratchets. | TC-025; TC-029; tests/shared_assurance.rs |
| FND-5002 | low | No remaining Rust review defect found in the bounded property slice. | tests/property.rs |

## Review

| Check | Result |
| --- | --- |
| Real boundary | Both properties call public `rewrite` and `check_equivalence`; no mock or local reimplementation substitutes for either production boundary. |
| Oracle independence | Rewrite rule selection/application and exact trace evaluation are separate implementations. The oracle evaluates the pre/post graphs rather than asserting only that the rewriter returned its own expected status. |
| Domain and bounds | Temporal kinds, Boolean families and interval endpoints are generated from explicit finite domains. The 32-case cap and interval ceiling are visible and no universal claim beyond them is made. |
| Mutation sensitivity | A missing rule step, wrong normalized status, unequal trace semantics, or omitted Boolean family changes a direct assertion or the generated population. |
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

This review accepts the property-test increment only. It does not claim fuzz
execution, mutation effectiveness, universal temporal equivalence, production
monitor qualification or closure of the cross-repository TL effectiveness
epic.
