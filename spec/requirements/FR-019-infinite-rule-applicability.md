---
id: FR-019
title: Admit only profile-proved infinite-trace rewrite rules
type: FR
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-001
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/FR-009
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-289
    type: depends_on
---

# FR-019: Admit only profile-proved infinite-trace rewrite rules

## Description

When an infinite-trace graph is submitted, tl-rewrite shall select a distinct
`mltl.infinite-trace/v1` rule catalog and apply only rules whose equations are
valid for the selected bounded or unbounded interval shape and the provider's
possibility-set semantics.

## Inputs

- A strict-read `tl-syntax.formula-unbounded/v1` graph, exact profile and
  event-position clock, optional fairness premises, and rewrite limits.

## Outputs

- A same-edition, same-profile graph with remapped fairness roots and an exact
  ordered rule trace, or a typed refusal with no partial output.

## Behavior

The catalog lists every rule identity and interval precondition explicitly.
Existing finite and origin-history catalogs retain their bytes and rule
ordering. An infinite request never falls back to either catalog. A rule's
admission requires a derivation over each complete Boolean refinement of a
partial valuation; identical proposition occurrences share one refinement.
For a rewrite-then-evaluate check, conflicting trace evidence is classified
before either formula is evaluated, so an algebraic fold cannot hide a
provider `failed` disposition. A graph-only rewrite does not inspect traces.

| Existing rule family | Infinite-profile admission |
|---|---|
| `bool.*` | Enabled with the same operands and priority, because the Boolean equations hold for every common complete refinement; no new rule is implied. |
| `neg.future.dual`, `neg.globally.dual`, `neg.until.dual`, `neg.release.dual` | Enabled for either closed or unbounded intervals, preserving that exact interval variant. |
| `temporal.future.false/true`, `temporal.globally.false/true`, `temporal.until.true-left`, `temporal.release.false-left` | Enabled for either closed or unbounded intervals. |
| `temporal.future/globally/until/release.singleton` | Enabled only for the closed `[0,0]` interval. An unbounded interval never satisfies a singleton precondition. |
| `past.once.strong-previous` | Enabled only for closed `[1,1]` at the exact origin-complete event-position clock. |
| `past.triggered.fold-dual` | Enabled for closed or unbounded `S/T` intervals with exact operand identity. |
| `west.nested-until-right`, `west.nested-release-right` | Excluded pending the already-declared decomposition and growth proof. |
| Any other rule or an unsupported operator/interval combination | Refused; there is no wildcard fallback to a Boolean or finite rule. |

The future and past duals are valid pointwise at each complete infinite trace;
lifting them over the same set of refinements preserves the result set. The
existing finite-trace horizon argument is not reused as their proof. Rule
enablement also requires the independent oracle checks in FR-020. Fairness
premise roots are rewritten through the same graph, retaining premise order,
identity correspondence and the original lasso-admission condition. If a
root cannot be mapped exactly, the whole request refuses.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-019-AC-1 | Each catalog family and interval form has its stated enabled/excluded disposition; a forbidden family or unbounded singleton never executes. | Test (TC-067) |
| FR-019-AC-2 | Every output retains owner edition, profile, event-position clock and remapped ordered fairness roots; finite and past catalog bytes remain unchanged. | Test (TC-068) |
| FR-019-AC-3 | A conformance check classifies conflicting valuations before evaluation; unknown rules, foreign profile/clock or unmappable premises refuse before a partial graph or positive evidence. | Test (TC-069) |

## Dependencies

FR-001 supplies rule-catalog identity; FR-009 supplies the existing past rules;
tl-syntax FR-289 supplies the separate unbounded document.
