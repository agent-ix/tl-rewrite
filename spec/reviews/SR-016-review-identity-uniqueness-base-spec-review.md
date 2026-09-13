---
id: SR-016
title: "Base specification review — tracked SpecReview identity uniqueness"
type: SpecReview
analysis: base
scope: "agent-ix/tl-rewrite#30; NFR-003-AC-6; TC-037; spec/reviews"
review_set: base
relationships:
  - target: ix://agent-ix/tl-rewrite/NFR-003
    type: reviews
---

# SR-016: Base specification review — tracked SpecReview identity uniqueness

## Summary

The user-selected base review checked identity allocation, requirement grammar,
failure and edge cases, criterion-to-test traceability, and the merge order for
open rewrite branches. This PR must land first with SR-015 and SR-016; every
other open branch then rebases and allocates above that range.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-1601 | high | Main contains two tracked artifacts claiming SR-012, while strict Quire validation succeeds. The residual review is reallocated to first-free SR-015 and the contract adds a tracked uniqueness census. | SR-012, SR-015, NFR-003-AC-6, TC-037 |
| FND-1602 | high | PR #24 and three sibling branches allocate overlapping SR-015 through SR-018 without Git conflicts. Landing the census first converts every later collision into a red local gate; those branches must rebase and renumber. | tl-rewrite#24, tl-rewrite#28, tl-rewrite#29, tl-rewrite#30 |
| FND-1603 | medium | Comparing raw frontmatter lines can treat `SR-091` and `"SR-091"` as different, while an empty scan can pass vacuously. Both are explicit negative controls. | NFR-003-AC-6, TC-037 |

## Dispositions

All findings are resolved at specification level. Implementation is limited to
first-party Rust in the existing shared-assurance test target and the sealed
Quire population totals changed by one criterion and one matrix row. No parser,
rewrite behavior, dependency, hosted trigger, release authority, or generic
assurance framework is authorized.
