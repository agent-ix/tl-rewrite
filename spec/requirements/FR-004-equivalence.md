---
id: FR-004
title: Validate bounded rewrite equivalence
type: FR
relationships:
  - target: ix://agent-ix/tl-rewrite/StR-002
    type: implements
---

# FR-004: Validate bounded rewrite equivalence

## Description

The qualification suite shall compare original and rewritten closed-trace
formulas with the independent, dev-only `tl-oracle` over every valuation in a
declared horizon-complete finite domain and shall retain permitted
WEST-derived fixtures. The public runtime equivalence API may use the pinned
tl-mltl evaluator to provide a bounded diagnostic, but that self-comparison
is not evidence that enables a rewrite rule or proves semantic soundness.

## Behavior

- The trace length is one plus the maximum checked lookahead of the formula pair.
- Enumeration covers every valuation of every referenced proposition at every
  instant unless a configured proposition, horizon, trace, or work bound makes
  the result non-conclusive.
- A first mismatch retains a deterministic counterexample trace and both verdicts.
- Reports name formula, profile, rule set, trace domain, oracle or diagnostic
  evaluator role, syntax,
  WEST source/corpus, and result status.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-004-AC-1 | Supported small formula pairs receive exhaustive horizon-complete agreement against the independent oracle or a retained minimal-by-enumeration counterexample; the production evaluator is not the qualification authority. | Test (TC-013, TC-014, TC-066) |
| FR-004-AC-2 | Resource-limited, profile-incompatible, or evaluator-error cases serialize as non-conclusive and are absent from positive rule-enablement evidence. | Test (TC-015) |
| FR-004-AC-3 | The pinned MIT-licensed WEST validation subset and independent rule fixtures retain exact source, license, formula, catalog, evaluator, domain, and output digests. | Test (TC-016) |

## Dependencies

Depends on FR-002, the dev-only `tl-oracle` crate for qualification, exact
tl-mltl evaluation/horizon APIs for existing public diagnostics, and permitted
WEST artifacts. `tl-oracle` depends only on tl-syntax and shares no evaluator
implementation with tl-mltl.
