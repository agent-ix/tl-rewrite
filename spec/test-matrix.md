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
| FR-002 | FR-002-AC-1 through FR-002-AC-4 | TC-005 through TC-008, TC-020, TC-021, TC-037 | ✅ covered |
| FR-003 | FR-003-AC-1 through FR-003-AC-3 | TC-009 through TC-012 | ✅ covered |
| FR-004 | FR-004-AC-1 through FR-004-AC-3 | TC-013 through TC-016, TC-028 | ✅ covered |
| FR-005 | FR-005-AC-1, FR-005-AC-3 | TC-017, TC-019 | ✅ covered |
| FR-006 | FR-006-AC-1 through FR-006-AC-3, FR-006-AC-5 through FR-006-AC-8 | TC-023, TC-024, TC-025, TC-027, TC-028, TC-029, TC-038 | ✅ implemented |
| FR-007 | FR-007-AC-1 through FR-007-AC-6 | TC-031 through TC-036 | ✅ covered |

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Coverage Status |
|---|---|---|---|
| StR-001 | FR-001, FR-003, FR-005 | TC-001, TC-009, TC-010, TC-019 | ✅ covered |
| StR-002 | FR-002, FR-004, FR-006 | TC-005, TC-006, TC-013, TC-016, TC-028 | ✅ covered |
| StR-003 | FR-007 | TC-031 through TC-034 | ✅ covered |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-001 | deterministic and resource-bound tests | TC-004 through TC-008, TC-015, TC-017, TC-021, TC-031, TC-035 | ✅ covered |
| NFR-002 | catalog, corpus, context, and provenance inspection | TC-001, TC-002, TC-016, TC-019, TC-030, TC-031, TC-034, TC-036 | ✅ covered |
| NFR-003 | producer-boundary, outcome-distinguishability, identity, and mutation probes | TC-024, TC-027, TC-030, TC-037, TC-039 | ✅ implemented |

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
| TC-029 | Leave no local evidence framework or retained legacy evidence; enumerate every tracked and untracked-not-ignored repository path through Git; constrain exact denial, area, exact-declaration and reviewed historical-prose populations; scan raw bytes; fail closed on enumeration or reads; retain preferred-Makefile, non-UTF-8, unreadable-path, hostile-exemption, stable-disclosure and literal-`ci` controls. These are executable controls owned by FR-006-AC-7, not fields in a local assurance record. | Integration | P0 | FR-006-AC-7 | ✅ implemented |
| TC-030 | Require published revision constants to be the resolved revisions | Integration | P0 | NFR-002-AC-2, NFR-003-AC-5 | ✅ implemented |
| TC-031 | Preserve exact shared requirement context, signal-catalog identity, and distinct clause/node spans in contextual rewrite reports | Integration | P0 | FR-007-AC-1, StR-003-VC-1, NFR-001-AC-1, NFR-002-AC-3 | ✅ implemented |
| TC-032 | Verify exact contextual replay and reject every independently mutated, omitted, or substituted catalog/context/request input | Integration | P0 | FR-007-AC-2, StR-003-VC-2 | ✅ implemented |
| TC-033 | Validate input and output proposition bindings and return typed locus-specific non-success without successful output | Integration | P0 | FR-007-AC-3, StR-003-VC-2 | ✅ implemented |
| TC-034 | Bind context into bounded-equivalence reports and distinguish original/rewritten binding refusal before enumeration | Integration | P0 | FR-007-AC-4, StR-003-VC-1, NFR-002-AC-3 | ✅ implemented |
| TC-035 | Preserve context-free API outcomes and exact v1 bytes while strictly round-tripping contextual v2 report families | Snapshot | P0 | FR-007-AC-5, NFR-001-AC-1 | ✅ implemented |
| TC-036 | Emit contextual native domain records through the existing producer-owned shared intake boundary without new generic machinery | Integration | P0 | FR-007-AC-6, NFR-002-AC-3 | ✅ implemented |
| TC-037 | Refuse duplicate or vacuously empty tracked SpecReview identity populations, including equal plain and quoted YAML spellings | Integration | P0 | NFR-003-AC-6 | ✅ implemented |
| TC-038 | Restore exact tracked shared-input bytes after successful execution and forced spawn-failure unwind, and preserve the original panic during a forced scratch-path restoration failure | Integration | P0 | FR-006-AC-8 | ✅ implemented |
| TC-039 | Enforce one scoped registry ix-flow specification across semantic job-step run scalars, command-position bare/path-qualified npm after assignments, shell groups, the complete documented npm-install alias family, and literal nested shell commands; fail closed on unquoted redirection and executable expansion scripts; reject alternate/non-literal families and automatic triggers; ignore inert command arguments and non-`-c` shell invocations without suppressing later commands; verify the exact runtime | Integration | P0 | NFR-003-AC-7, NFR-003-AC-8, NFR-003-AC-9, NFR-003-AC-10, NFR-003-AC-11, NFR-003-AC-12 | ✅ implemented |
| TC-040 | Intern and identify span-distinct equivalent formula inputs semantically | Unit | P0 | FR-002-AC-4 | ✅ implemented |
