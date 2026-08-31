---
id: PLAN-001
title: tl-rewrite v0.1 implementation plan
type: Plan
status: active
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-001
    type: references
  - target: ix://agent-ix/tl-rewrite/FR-005
    type: references
---

# tl-rewrite v0.1 implementation plan

## Dependency DAG

```text
PGM-01 + exact tl-syntax + tl-mltl evaluator/horizon
  -> requirements, matrix, assurance packet, composite review
  -> primary-source and derived rule catalog
  -> deterministic bounded engine
  -> replayable versioned traces
  -> exhaustive bounded equivalence + pinned WEST fixtures
  -> retained evidence + independent human review
```

## Work Packages

1. Validate the specification, matrix, assurance packet, and composite review.
2. Implement the catalog and canonical catalog digest.
3. Implement bounded stable-order rewriting and explicit non-success states.
4. Implement step-chain recording and exact replay verification.
5. Implement tl-mltl-backed exhaustive comparison and WEST fixture checks.
6. Complete code/gap reviews and retain a PGM-01 evidence envelope without publishing.

## Exit Criteria

Every matrix row is backed, local and remote gates pass, enabled rules have
complete provenance and positive bounded evidence, non-conclusive states remain
explicit, and the human source-release decision remains open for independent review.
