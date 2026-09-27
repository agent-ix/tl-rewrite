---
id: FR-020
title: Qualify rewrite soundness with the independent oracle
type: FR
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-004
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/FR-019
    type: depends_on
---

# FR-020: Qualify rewrite soundness with the independent oracle

## Description

When a rule is proposed for bounded, past or infinite profiles, the
qualification suite shall compare its before/after results using the dev-only
`tl-oracle`, independently of tl-mltl and any infinite-trace provider.

## Inputs

- Exact rule/catalog revision, owner graph and profile, trace family,
  complete or partial valuations, fairness premises, oracle revision, and
  resource limits.

## Outputs

- Per-rule, per-profile agreement evidence or a minimal reproducible
  counterexample; resource-incomplete and unsupported cases remain non-passing.

## Behavior

The oracle is a separate unpublished crate depending only on tl-syntax. Its
evaluation code and lasso procedure share no production evaluator function or
internal module. Bounded exhaustive checks, origin-boundary past cases,
infinite lassos, fair/unfair loops, partial valuation refinements and finite
prefixes are separate populations. For every tested trace, the original and
rewritten formula must have the same FR-341 result and the same missing versus
conflicting distinction. A provider result is useful differential evidence,
but cannot serve as the oracle for the rule it implements.

A finite test population is evidence, not a universal proof. Rule enablement
also needs the catalog's semantic derivation; a mismatch disables the exact
rule/profile pair until resolved. Fuzzing rewrites then evaluates both sides
with the oracle; a crash, disagreement, changed corpus digest or skipped
profile is a failure. Reproducible seeds and minimized witnesses are retained
through the shared assurance path, with no local evidence store.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-020-AC-1 | Each enabled rule family is checked on bounded, origin-past and applicable infinite/partial populations by the independent oracle; wrong rules yield a concrete counterexample. | Test (TC-066, TC-070) |
| FR-020-AC-2 | Oracle dependency inspection proves no production evaluator import, and production tl-rewrite has no `tl-oracle` dependency edge. | Test (TC-071) |
| FR-020-AC-3 | Rewrite-then-evaluate fuzzing reaches real rewrite and oracle paths, records seed/revision/budget, and fails on disagreement, crash or skipped applicable case. | Test (TC-072) |

## Dependencies

TL-245 locates the `tl-oracle` crate in its own repo. TL-35 provides the
independent implementation before this qualification evidence can be green.
