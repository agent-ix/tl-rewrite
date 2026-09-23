---
id: NFR-005
title: Preserve rewrite identity and bounded work on infinite inputs
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-019
    type: constrains
  - target: ix://agent-ix/tl-rewrite/FR-020
    type: constrains
  - target: ix://agent-ix/tl-rewrite/FR-021
    type: constrains
---

# NFR-005: Preserve rewrite identity and bounded work on infinite inputs

## Statement

When an infinite formula is rewritten or checked, tl-rewrite shall preserve
exact owner/profile/fairness identities and stop at the declared iteration,
node, application, work, report and replay limits without a partial output.

## Scope

Applies to the new infinite catalog and its qualification path. Existing
finite and past limits and bytes remain unchanged.

## Rationale

An unbounded temporal interval does not authorize unbounded graph expansion
or a status that appears conclusive after a budget expires.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| One-over-limit operations returning a successful rewritten graph | 0 | 0 | Test (TC-075) |
| Exact-identity mutations accepted during replay | 0 | 0 | Test (TC-075) |

## Verification

Exercise exact and one-over limits separately, then mutate profile, edition,
clock, formula digest, catalog revision and premise-root mapping one at a time.
Repeat a fixed case to compare serialized output bytes.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-005-AC-1 | Exact limits pass; one-over limit or one-axis identity mutation refuses with no partial graph, report or positive conformance result. | Test (TC-075) |

## Dependencies

NFR-001 supplies existing deterministic work budgets; tl-syntax owns the
strict graph and fairness identity boundaries.
