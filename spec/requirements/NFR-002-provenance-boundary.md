---
id: NFR-002
title: Retain provenance and qualification boundaries
type: NFR
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---

# NFR-002: Retain provenance and qualification boundaries

## Statement

Every rule and exchanged record shall identify its semantic derivation or
primary source, schema, source/dependency revisions, corpus license and digest,
and human decision boundary.

## Scope

Rule metadata, wire records, copied corpus material, evidence, and release claims are in scope.

## Rationale

Semantic or provenance drift invalidates rewrite evidence even when selected fixtures still pass.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Enabled rules without complete provenance | 0 | 0 | Inspection |
| Unidentified exchanged or copied artifacts | 0 | 0 | Test |

## Verification

Catalog tests inspect mandatory identities, licenses, exclusions, and open
human decisions. `scripts/check_provenance.py` re-derives the retained corpus
digests and requires the published revision constants to be the revisions
`Cargo.toml` and `Cargo.lock` resolve, so a wire field cannot attribute a
verdict to a dependency that did not produce it.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-002-AC-1 | No rule is enabled without complete provenance, applicability, and revision metadata. | Test (TC-001, TC-002) |
| NFR-002-AC-2 | Every exchanged record names the exact tl-syntax, tl-mltl, WEST, and catalog identities the run used, those constants agree with what Cargo resolved, and no record claims universal proof or release. | Test (TC-016, TC-019, TC-030) |

## Dependencies

Applies PGM-01 to the complete repository lifecycle.
