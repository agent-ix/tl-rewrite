---
id: StR-003
title: Preserve temporal input context through rewriting
type: StR
---

# StR-003: Preserve temporal input context through rewriting

## Stakeholder Need

Temporal-tool developers and assurance reviewers need a rewrite, replay, or
bounded-equivalence result to remain attributable to the exact shared signal
catalog and, when supplied, the exact requirement revision and clause source
that the caller associated with the formula.

## Rationale

A formula digest and a node span identify syntax but do not identify the named
signals, requirement revision, clause, or source anchor whose meaning the
formula represents. Reusing a valid result with substituted or omitted context
would make deterministic replay semantically misleading even if the formula
bytes were unchanged.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-003-VC-1 | Context-aware rewrite, replay, and conformance records carry the exact shared requirement context when present, a digest identity of the complete shared signal catalog, and keep clause spans distinct from per-step formula-node spans. | Test (TC-031, TC-034) |
| StR-003-VC-2 | Replay rejects a changed, omitted, or substituted signal catalog or caller context, and no successful contextual result contains an unresolved proposition binding. | Test (TC-032, TC-033) |

## Stakeholders

Temporal frontend developers, consuming monitor projects, assurance reviewers,
and the human source-release owner.

## Context and Assumptions

The signal catalog and optional requirement context are the shared validated
types defined by the exact `tl-syntax` revision. tl-rewrite preserves and binds
those values; it does not infer them, validate whether their provenance claims
are true, or translate a contract IR.

## Traceability

This need is realized by FR-007 and verified by TM-001.
