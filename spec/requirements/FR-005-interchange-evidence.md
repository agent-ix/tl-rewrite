---
id: FR-005
title: Preserve versioned interchange and evidence boundaries
type: FR
---

# FR-005: Preserve versioned interchange and evidence boundaries

## Description

All catalog, rewrite, replay, and conformance records shall use closed v1 wire
identities and the candidate shall retain a canonical PGM-01 evidence envelope.

## Behavior

- Unknown schema identities and omitted material fields are rejected.
- Evidence pins source, policy/schema, toolchain, dependencies, catalogs,
  corpora, parameters, commands, outputs, and limitations.
- Failed and non-conclusive outcomes are retained separately from passes.
- Automation records no release, qualification, accreditation, or certification decision.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-005-AC-1 | Round trips preserve every v1 field and reject unknown schemas or missing identities. | Test (TC-017) |
| FR-005-AC-2 | The evidence collector refuses overwrite, retains command failures, and validates local schemas plus the exact merged PGM-01 envelope. | Test (TC-018) |
| FR-005-AC-3 | Evidence and documentation preserve human authority and bounded-equivalence limitations without an automated release or qualification claim. | Inspection (TC-019) |

## Dependencies

Packages FR-001 through FR-004 under PGM-01 without changing semantic outcomes.
