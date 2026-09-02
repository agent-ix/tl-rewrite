---
id: FR-005
title: Preserve versioned interchange and evidence boundaries
type: FR
---

# FR-005: Preserve versioned interchange and evidence boundaries

## Description

All catalog, rewrite, replay, and conformance records shall use closed v1 wire
identities, and the evidence this repository has already retained shall remain
immutable and readable.

## Behavior

- Unknown schema identities and omitted material fields are rejected.
- Every wire record names the exact syntax, evaluator, WEST, and catalog
  identities the run actually used, and a bounded comparison carries the
  limitation describing the domain it enumerated.
- Failed and non-conclusive outcomes are retained separately from passes.
- The eleven records retained under `evidence/` are immutable. Retention,
  integrity checking, audit, attestation and receipt are owned upstream by Quoin
  from FR-006 onward; this repository keeps the records, not a verifier for
  them.
- Automation records no release, qualification, accreditation, or certification
  decision.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-005-AC-1 | Round trips preserve every v1 field and reject unknown schemas or missing identities. | Test (TC-017) |
| FR-005-AC-2 | Every byte of the eleven retained evidence records is unchanged, read through the shared compatibility mapping rather than a local verifier, and the mapping's answer is reported as it stands. | Test (TC-026) |
| FR-005-AC-3 | Evidence and documentation preserve human authority and bounded-equivalence limitations without an automated release or qualification claim. | Inspection (TC-019) |

## Dependencies

Packages FR-001 through FR-004 without changing semantic outcomes, and hands
retention and integrity to the shared intake path defined by FR-006.
