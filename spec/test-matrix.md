---
id: TM-001
title: tl-rewrite v0.1 test matrix
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-rewrite/MRS-001
    type: covers
---

# tl-rewrite v0.1 Test Matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| FR-001 | FR-001-AC-1 through FR-001-AC-3 | TC-001 through TC-004 | ✅ covered |
| FR-002 | FR-002-AC-1 through FR-002-AC-3 | TC-005 through TC-008, TC-020, TC-021 | ✅ covered |
| FR-003 | FR-003-AC-1 through FR-003-AC-3 | TC-009 through TC-012 | ✅ covered |
| FR-004 | FR-004-AC-1 through FR-004-AC-3 | TC-013 through TC-016, TC-028 | ✅ covered |
| FR-005 | FR-005-AC-1, FR-005-AC-3 | TC-017, TC-019 | ✅ covered |
| FR-006 | FR-006-AC-1 through FR-006-AC-3, FR-006-AC-5 through FR-006-AC-7 | TC-023, TC-024, TC-025, TC-027, TC-028, TC-029 | ✅ covered |

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Coverage Status |
|---|---|---|---|
| StR-001 | FR-001, FR-003, FR-005 | TC-001, TC-009, TC-010, TC-019 | ✅ covered |
| StR-002 | FR-002, FR-004, FR-006 | TC-005, TC-006, TC-013, TC-016, TC-028 | ✅ covered |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-001 | deterministic and resource-bound tests | TC-004 through TC-008, TC-015, TC-017, TC-021 | ✅ covered |
| NFR-002 | catalog, corpus, and provenance inspection | TC-001, TC-002, TC-016, TC-019, TC-030 | ✅ covered |
| NFR-003 | producer-boundary, outcome-distinguishability, and mutation probes | TC-024, TC-027, TC-030 | ✅ covered |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-001 | Validate enabled rule metadata and uniqueness | Unit | P0 | FR-001-AC-1, NFR-002-AC-1 | ✅ implemented |
| TC-002 | Retain excluded rule dispositions and primary-source pins | Unit | P0 | FR-001-AC-1, NFR-002-AC-1 | ✅ implemented |
| TC-003 | Apply primitive and nested catalog derivations | Unit | P0 | FR-001-AC-2 | ✅ implemented |
| TC-004 | Detect catalog revision drift | Unit | P0 | FR-001-AC-3, NFR-001-AC-1 | ✅ implemented |
| TC-005 | Repeat rewrite reports byte-identically | Unit | P0 | FR-002-AC-1, NFR-001-AC-1 | ✅ implemented |
| TC-006 | Exhaust iteration, application, and logical-work budgets | Unit | P0 | FR-002-AC-2, NFR-001-AC-2 | ✅ implemented |
| TC-007 | Reject node growth and checked-address limits | Unit | P0 | FR-002-AC-2, NFR-001-AC-2 | ✅ implemented |
| TC-008 | Preserve profile, graph validity, and fixed point | Unit | P0 | FR-002-AC-3 | ✅ implemented |
| TC-009 | Record every output-changing rule step | Unit | P0 | FR-003-AC-1 | ✅ implemented |
| TC-010 | Replay an exact successful trace | Unit | P0 | FR-003-AC-2, StR-001-VC-2 | ✅ implemented |
| TC-011 | Reject changed replay inputs and intermediates | Unit | P0 | FR-003-AC-2 | ✅ implemented |
| TC-012 | Separate partial diagnostics from successful output | Unit | P0 | FR-003-AC-3 | ✅ implemented |
| TC-013 | Exhaustively confirm supported equivalence pairs | Integration | P0 | FR-004-AC-1 | ✅ implemented |
| TC-014 | Retain deterministic equivalence counterexamples | Integration | P0 | FR-004-AC-1 | ✅ implemented |
| TC-015 | Keep bounded-resource and profile cases non-conclusive | Integration | P0 | FR-004-AC-2, NFR-001-AC-2 | ✅ implemented |
| TC-016 | Exercise pinned WEST and independent fixtures | Integration | P0 | FR-004-AC-3, StR-002-VC-2, NFR-002-AC-2 | ✅ implemented |
| TC-017 | Round trip and reject versioned wire records | Integration | P0 | FR-005-AC-1, NFR-001-AC-1 | ✅ implemented |
| TC-019 | Inspect human authority and qualification boundary | Integration | P0 | FR-005-AC-3, StR-001-VC-1 | ✅ implemented |
| TC-020 | Detect repeated complete states through the rewrite engine | Unit | P0 | FR-002-AC-2, NFR-001-AC-2 | ✅ implemented |
| TC-021 | Charge retained-source provenance traversal to the work budget | Unit | P0 | FR-002-AC-2, NFR-001-AC-3 | ✅ implemented |
| TC-022 | Bind the rule corpus to the constructed reviewed fixtures | Integration | P0 | FR-001-AC-2, FR-004-AC-3 | ✅ implemented |
| TC-023 | Classify every shared pin through the packaged matrix | Integration | P0 | FR-006-AC-1 | ✅ implemented |
| TC-024 | Reach Quoin without Quoin or Quire executing a producer | Integration | P0 | FR-006-AC-2, NFR-003-AC-1, NFR-003-AC-2 | ✅ implemented |
| TC-025 | Bind the sealed record's impact snapshot to the Quire export | Integration | P0 | FR-006-AC-3 | ✅ implemented |
| TC-027 | Demonstrate all twelve verification outcomes with paired controls | Integration | P0 | FR-006-AC-5, NFR-003-AC-3 | ✅ implemented |
| TC-028 | Retain every counterexample as an independently replayed witness | Integration | P0 | FR-006-AC-6, FR-004-AC-1, StR-002-VC-2 | ✅ implemented |
| TC-029 | Leave no local evidence framework or retained legacy evidence; enumerate every tracked and untracked-not-ignored repository path through Git, constrain exact denial, area and declaration-exemption sets, scan raw bytes, fail closed on enumeration or reads, and retain preferred-Makefile, non-UTF-8, unreadable-path, hostile-exemption and Make-disclosure controls | Integration | P0 | FR-006-AC-7 | ✅ implemented |
| TC-030 | Require published revision constants to be the resolved revisions | Integration | P0 | NFR-002-AC-2, NFR-003-AC-5 | ✅ implemented |
