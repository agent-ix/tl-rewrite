---
id: FR-005
title: Preserve versioned interchange boundaries
type: FR
---

# FR-005: Preserve versioned interchange boundaries

## Description

All catalog, rewrite, replay, and conformance records shall use closed v1 wire
identities.

## Behavior

- Unknown schema identities and omitted material fields are rejected.
- Every wire record names the exact syntax, evaluator, WEST, and catalog
  identities the run actually used, and a bounded comparison carries the
  limitation describing the domain it enumerated.
- Failed and non-conclusive outcomes are retained separately from passes.
- Automation records no release, qualification, accreditation, or certification
  decision.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-005-AC-1 | Round trips preserve every v1 field and reject unknown schemas or missing identities. | Test (TC-017) |
| FR-005-AC-3 | Evidence and documentation preserve human authority and bounded-equivalence limitations without an automated release or qualification claim. | Inspection |

## Dependencies

Packages FR-001 through FR-004 without changing semantic outcomes.
