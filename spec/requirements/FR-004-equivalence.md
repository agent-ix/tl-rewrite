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

The library shall compare original and rewritten closed-trace formulas with the
exact tl-mltl evaluator over every valuation in a declared horizon-complete
finite domain and shall retain permitted WEST-derived fixtures.

## Behavior

- The trace length is one plus the maximum checked lookahead of the formula pair.
- Enumeration covers every valuation of every referenced proposition at every
  instant unless a configured proposition, horizon, trace, or work bound makes
  the result non-conclusive.
- A first mismatch retains a deterministic counterexample trace and both verdicts.
- Reports name formula, profile, rule set, trace domain, evaluator, syntax,
  WEST source/corpus, and result status.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-004-AC-1 | Supported small formula pairs receive exhaustive horizon-complete agreement or a retained minimal-by-enumeration counterexample. | Test (TC-013, TC-014) |
| FR-004-AC-2 | Resource-limited, profile-incompatible, or evaluator-error cases serialize as non-conclusive and are absent from positive rule-enablement evidence. | Test (TC-015) |
| FR-004-AC-3 | The pinned MIT-licensed WEST validation subset and independent rule fixtures retain exact source, license, formula, catalog, evaluator, domain, and output digests. | Test (TC-016) |

## Dependencies

Depends on FR-002, exact tl-mltl evaluation/horizon APIs, and permitted WEST artifacts.
