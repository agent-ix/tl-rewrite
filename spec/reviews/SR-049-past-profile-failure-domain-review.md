---
id: SR-049
title: "Failure-domain review — past-profile rewrites"
type: SpecReview
analysis: failure-domain
scope: "FR-009 and its FR-001/FR-002/FR-003/FR-007 plus tl-syntax FR-011/FR-013 boundaries"
review_set: all
---

## Summary

Reviewed profile/catalog identity, graph topology, budget failure, contextual
binding, replay substitution, and external dependency/corpus boundaries. All
identified failure domains are now explicit and have negative coverage.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-4901 | high | FIXED: rebuilding through the legacy constructor could silently down-convert formula-v2 or turn a valid past graph into an internal failure; FR-009 now requires schema preservation and TC-047 covers every O/H/Y/S/T rebuild path. | FR-009-AC-2, TC-047 |
| FND-4902 | medium | FIXED: invalid, mixed, unknown, online, unproved, unresolved-binding, and budget-exhausted domains are distinguished; no partial transformed graph may escape. | FR-009-AC-5, TC-050, TC-051 |
| FND-4903 | low | No callback, plugin, mutable entity, cyclic traversal, or side-effecting evaluation extension is introduced. Existing validated topology, checked node/work limits, deterministic interning, and immutable report identities bound the remaining domains. | FR-009 Error and Boundary Conditions |
