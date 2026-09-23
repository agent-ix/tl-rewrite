---
id: TM-002
title: Infinite-trace rewrite soundness test matrix
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-rewrite/MRS-001
    type: covers
---

# Infinite-trace rewrite soundness test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| FR-019 | FR-019-AC-1 through FR-019-AC-3 | TC-067 through TC-069 | 🚧 planned |
| FR-020 | FR-020-AC-1 through FR-020-AC-3 | TC-066, TC-070 through TC-072 | 🚧 planned |
| FR-021 | FR-021-AC-1 through FR-021-AC-2 | TC-073, TC-074 | 🚧 planned |
| NFR-005 | NFR-005-AC-1 | TC-075 | 🚧 planned |

TC-066 begins after the 0.3.0 release's implemented TC-065. The
`spec/stubs/v1_spec_stubs.rs` red lane is a specification aid and supplies no
coverage; TL-16/TL-13 replace each placeholder with tests of production
rewrite and independent-oracle paths.

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-066 | Distinguish runtime diagnostic comparison from independent rule evidence | Integration | P0 | FR-004-AC-1, FR-010-AC-4, FR-020-AC-1 | 🚧 planned |
| TC-067 | Dispatch each rule family and interval form by exact infinite profile | Property | P0 | FR-010-AC-6, FR-019-AC-1 | 🚧 planned |
| TC-068 | Preserve formula edition, clock and remapped fairness roots | Property | P0 | FR-019-AC-2 | 🚧 planned |
| TC-069 | Refuse conflict, unknown rule and foreign identities before mutation | Integration | P0 | FR-019-AC-3 | 🚧 planned |
| TC-070 | Find counterexamples for wrong rules on bounded, past, lasso and partial inputs | Property | P0 | FR-020-AC-1 | 🚧 planned |
| TC-071 | Inspect separate oracle dependency closure and absence from production | Inspection | P0 | FR-020-AC-2 | 🚧 planned |
| TC-072 | Rewrite-then-evaluate fuzz target with real oracle and checked seeds | Fuzz | P0 | FR-020-AC-3 | 🚧 planned |
| TC-073 | Exhaustively map statuses to FR-341 without message matching | Unit | P0 | FR-021-AC-1 | 🚧 planned |
| TC-074 | Reject replay promotion of unsupported, incomplete or failed checks | Integration | P0 | FR-021-AC-2 | 🚧 planned |
| TC-075 | Exercise each exact and one-over bound and one-axis identity mutation | Property | P0 | NFR-005-AC-1 | 🚧 planned |

## Integration Test Matrix

| Purpose | Target | Type | Test Cases |
|---|---|---|---|
| Compare rule semantics independently | tl-oracle | workspace | TC-066, TC-070 through TC-072 |
| Preserve owner graph and fairness identity | tl-syntax corpus | workspace | TC-067 through TC-069 |
| Refuse non-conclusive promotion | rewrite report and replay | workspace | TC-073 through TC-075 |
