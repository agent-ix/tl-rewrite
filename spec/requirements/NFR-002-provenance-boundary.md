---
id: NFR-002
title: Retain provenance and qualification boundaries
type: NFR
---

# NFR-002: Retain provenance and qualification boundaries

## Statement

Every rule and exchanged record shall identify its semantic derivation or
primary source, schema, source revisions, corpus license,
and human decision boundary.

## Scope

Rule metadata, shared signal/context identities, wire records, copied corpus
material, evidence, and release claims are in scope.

## Rationale

Semantic or provenance drift invalidates rewrite evidence even when selected fixtures still pass.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Enabled rules without complete provenance | 0 | 0 | Inspection |
| Unidentified exchanged or copied artifacts | 0 | 0 | Test |

## Verification

Catalog tests inspect mandatory identities, licenses, exclusions, and open
human decisions.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-002-AC-1 | No rule is enabled without complete provenance, applicability, and revision metadata. | Test (TC-001, TC-002) |
| NFR-002-AC-2 | Every exchanged record names the exact WEST and catalog identities the run used, and no record claims universal proof or release. | Test (TC-016) |
| NFR-002-AC-3 | Every contextual native result preserves the exact supplied shared requirement context and identifies the complete supplied signal catalog without claiming that tl-rewrite validated the truth of either provenance statement. | Test (TC-031, TC-034) |

## Dependencies

Applies to the complete repository lifecycle.
