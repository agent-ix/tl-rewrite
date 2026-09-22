---
id: SR-067
title: "Rust review — bounded rewrite property baseline"
type: SpecReview
analysis: code-review
scope: "tests/property.rs, spec/test-matrix.md and assurance census pins at f672605 over origin/main 033a687, rebased onto origin/main cecb9f4"
review_set: subset
---

# Rust review — bounded rewrite property baseline

## Summary

Reviewed the issue-27 property increment after rebasing onto current main.
The branch adds one bounded check for the four reflexive Boolean rewrite
families and retains the existing bounded Until/Release property. The new
property exercises the public rewrite path and the independent exact
evaluator, and its test/matrix identity is repository-unique as TC-054 (moved
from TC-046, which the rebase's unrelated upstream work had since claimed for
a different past-profile control). Its closed family enum is exhaustively
iterated and the asserted rule-identity census prevents accidental
duplication.

Renumbered from SR-050 to SR-067 when rebasing onto current `main`, whose own
unrelated work had since claimed the SR-050 id (`agent-ix/tl-rewrite#27`).

## Verdict

**PASS** — the check exercises the correct public boundary and an independent
evaluator for every member of its four-family Boolean domain. The fuzz boundary
is a separate evidence-method finding in SR-066 and is not hidden by this
verdict.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-6705 | low | No remaining Rust review defect was found in the bounded property slice at `f672605`; every reflexive Boolean family is executed exactly once and checked through the public rewrite and independent equivalence boundaries. | `tests/property.rs`; TM-001 TC-054 |

## Remediation history

FND-6701 and FND-6703 were fixed during main integration and the initial full
gate. FND-6704 was fixed at `f672605` by replacing the randomized selector with
exhaustive enumeration and an asserted unique rule-identity census. The test
and matrix identity moved from TC-046 to TC-054, and this review from SR-050
to SR-067, while rebasing onto current `main` to resolve the id collision with
unrelated upstream work; no test content changed as part of that rename.

## Review

| Check | Result |
| --- | --- |
| Real boundary | Both properties call public `rewrite` and `check_equivalence`; no mock or local reimplementation substitutes for either production boundary. |
| Oracle independence | Rewrite rule selection/application and exact trace evaluation are separate implementations. The oracle evaluates the pre/post graphs rather than asserting only that the rewriter returned its own expected status. |
| Domain and bounds | Temporal kinds and interval endpoints are generated from explicit finite domains. The Boolean families are a closed four-variant enum and are exhaustively enumerated. |
| Mutation sensitivity | A missing family, duplicate rule identity, missing rule step, wrong normalized status or unequal trace semantics changes a direct assertion. |
| Panic surface | The interval constructor unwrap is over a strategy that constructs only `1 <= start <= end <= 3`; output unwrap follows an asserted `Normalized` status and remains test-only. Production panic/unsafe surface is unchanged. |
| Async, locks and resources | No async task, lock, I/O, unbounded generator, or production allocation path is added. Proptest disables persistence and uses the repository's fixed case count. |
| Trace integrity | The new property carries TC-054 and the three claimed criterion IDs. Current main's other TC ids remain distinct and intact. |
| Repository idioms | Existing `tests/property.rs` fixtures, public types, proptest configuration, formatting and stable-MSRV dependency set are reused; no dependency or gate changed. |

## Gates

The complete local `make ci` gate was rerun after the rebase onto `origin/main`
`cecb9f4`:

- formatting and all-target/all-feature Clippy with warnings denied: clean;
- 82/84 Rust tests pass (`cargo test --all-targets --all-features
  --no-fail-fast`), including both bounded property tests and the
  SpecReview-identity-uniqueness control (TC-037); the 2 non-passing are
  `every_shared_pin_is_classified_by_the_packaged_matrix` and
  `hosted_ix_flow_identity_and_manual_trigger_are_exact`, both pre-existing on
  `origin/main` `cecb9f4` itself (independently reproduced against a clean
  checkout) because the local `quire-cli` (0.32.2), `quoin`
  (0.23.1-94-g0128ed4) and `ix-flow` (0.2.3) exceed the versions the packaged
  compatibility matrix has accepted; the matrix reports each `unknown`
  (`pending_human_acceptance`), not `incompatible`. Unrelated to this branch's
  content;
- corpus provenance, rule conformance, counterexample replay and
  normalization: clean;
- cargo-deny, unsafe-comment audit, Rust 1.98.1 all-target/all-feature check
  and warning-free rustdoc: clean;
- `quire coverage`: 126/126 matrix rows backed (100%), 84/84/84 Rust
  candidates/tagged/bound;
- `make assurance` (`pins` + `assurance-chain`) fails at `pins` for the same
  pre-existing tool-version reason as above.

This review's boundary and mutation-sensitivity findings depend only on
`tests/property.rs`'s content, which is unchanged by the rebase apart from the
TC-046 → TC-054 rename.

## Boundary

This review clears the bounded Boolean family-completeness claim. It does not
claim fuzz execution, mutation effectiveness, universal temporal equivalence, production
monitor qualification or closure of the cross-repository TL effectiveness
epic.
