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
| FR-002 | FR-002-AC-1 through FR-002-AC-3 | TC-005 through TC-008 | ✅ covered |
| FR-003 | FR-003-AC-1 through FR-003-AC-3 | TC-009 through TC-012 | ✅ covered |
| FR-004 | FR-004-AC-1 through FR-004-AC-3 | TC-013 through TC-016 | ✅ covered |
| FR-005 | FR-005-AC-1 through FR-005-AC-3 | TC-017 through TC-019 | ✅ covered |

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Coverage Status |
|---|---|---|---|
| StR-001 | FR-001, FR-003, FR-005 | TC-001, TC-009, TC-010, TC-019 | ✅ covered |
| StR-002 | FR-002, FR-004 | TC-005, TC-006, TC-013, TC-016 | ✅ covered |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-001 | deterministic and resource-bound tests | TC-004 through TC-008, TC-015, TC-017 | ✅ covered |
| NFR-002 | catalog, corpus, and evidence inspection | TC-001, TC-002, TC-016, TC-018, TC-019 | ✅ covered |

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
| TC-018 | Verify immutable evidence contract and schemas | Integration | P0 | FR-005-AC-2, NFR-002-AC-2 | ✅ implemented |
| TC-019 | Inspect human authority and qualification boundary | Integration | P0 | FR-005-AC-3, StR-001-VC-1, NFR-002-AC-2 | ✅ implemented |
