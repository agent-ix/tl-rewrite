---
id: NFR-001
title: Preserve deterministic bounded resource behavior
type: NFR
---

# NFR-001: Preserve deterministic bounded resource behavior

## Statement

All semantic outputs shall depend only on identified inputs. Checked budgets,
node addressing, horizon arithmetic, and enumeration cardinality shall fail
before wraparound, uncontrolled growth, or an incomplete success claim.

## Scope

Catalog serialization, rewrite execution, replay, equivalence enumeration, and evidence are in scope.

## Rationale

Repeatability and honest resource failure are prerequisites for reviewable rewrite evidence.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Nondeterministic output mismatches | 0 | 0 | Test |
| Wrapped, partial, or unbounded success claims | 0 | 0 | Test |

## Verification

Requirement-tagged unit and integration tests repeat all public operations and
exercise every budget and checked-cardinality boundary.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-001-AC-1 | Repeated catalog, rewrite, replay, conformance, and serialization operations are byte-for-byte stable. | Test (TC-004, TC-005, TC-017) |
| NFR-001-AC-2 | Work, iteration, node, application, horizon, proposition, and trace-domain excess are explicit non-success outcomes. | Test (TC-006, TC-007, TC-015) |
| NFR-001-AC-3 | Retained-source provenance visits each source node at most once and charges each visit to the logical-work budget. | Test (TC-021) |

## Dependencies

Constrains FR-001 through FR-005.
