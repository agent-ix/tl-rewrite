---
id: StR-002
title: Bound execution and independently check equivalence
type: StR
---

# StR-002: Bound execution and independently check equivalence

## Stakeholder Need

Embedded and assurance consumers need deterministic termination controls and
explicit evidence that enabled rewrites agree with the named reference
semantics over a declared finite domain.

## Rationale

Cycles, uncontrolled graph growth, partial normalization, and sampled evidence
misreported as proof undermine both resource planning and semantic confidence.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-002-VC-1 | Identical input, catalog, strategy, and budgets produce identical success or explicit non-conclusive status. | Test (TC-005) |
| StR-002-VC-2 | Conformance reports identify the complete enumerated trace domain, evaluator, dependencies, catalog, and external corpus revisions. | Test (TC-016) |

## Stakeholders

Embedded temporal-tool developers, assurance reviewers, and corpus maintainers.

## Context and Assumptions

Bounded enumeration is conclusive only for the stated formula pair and finite domain.

## Traceability

This need is realized by FR-002 and FR-004 and verified by TM-001.
