---
id: FR-021
title: Keep infinite rewrite refusals and result labels distinct
type: FR
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-019
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/FR-020
    type: depends_on
---

# FR-021: Keep infinite rewrite refusals and result labels distinct

## Description

When an infinite rewrite or its conformance check cannot complete, tl-rewrite shall
return a typed status that retains the profile, rule and failure axis without
converting an unavailable check into a soundness claim.

## Inputs

- The exact owner formula, profile, optional fairness binding, rule catalog,
  limits and independent-oracle availability.

## Outputs

- An unchanged/no-applicable-rule, applied, unsupported, resource-incomplete,
  or failed result with exact source and catalog identities.

## Behavior

An unknown profile, foreign edition/clock, unsupported interval/operator or
absent oracle capability maps to FR-341 `unsupported`. Exhausted rewrite,
oracle or lasso work maps to FR-341 `inconclusive` with resource-incomplete
detail. Malformed owner bytes or an internal consistency failure maps to
`failed` or a pre-evaluation typed refusal, never `proved`. A successful
rewrite is not itself a temporal verdict; only a provider may later claim
`proved` or `refuted`. The mapping from every closed `RewriteStatus` and
`ConformanceStatus` variant is exhaustive, with no message parsing or wildcard
arm. Missing and conflicting valuations remain separate axes.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-021-AC-1 | Every status variant maps to one declared FR-341 label or no-verdict state and preserves the original typed reason. | Test (TC-073) |
| FR-021-AC-2 | Unsupported, resource-incomplete and failed outcomes cannot count as rule-enablement evidence or be promoted by serialization/replay. | Test (TC-074) |

## Dependencies

FR-019 owns the catalog; FR-020 owns independent soundness evidence.
