---
id: SR-014
title: Authorial residual-gap record for post-merge fixes
type: SpecReview
analysis: gap-analysis
scope: PLAN-003, issue #19, FR-006-AC-7, TC-027, TC-029
review_set: all
---

# SR-014: Authorial residual-gap record for post-merge fixes

## Summary

The implementation author recorded every repository-scoped residual in issue
#19 as covered by implementation or a retained control. The sole remaining gap
is the deliberately deferred common
Make execution-control qualification tracked outside this change.

## Authorship and authority

This is an authorial gap record for candidate
`d4670c25f3300a4859688e0af94f4fd995ba65af`, later landed on `main`. Its
coverage statements are implementation status, not independent review
clearance, approval, or merge authority. A separately authored review must
make any such determination.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-1401 | medium | DEFERRED. Full expanded Make execution-graph and command qualification remains open in `tl-rewrite#11` and `engineering-assurance#11`; this change does not add a local parser or claim closure. | NFR-003 | correct-requirement-no-evidence |
| FND-1402 | low | IMPLEMENTED (authorial record; independent clearance pending). All other issue #19 entries map to code, specification text, or retained review evidence in the closure matrix below. | PLAN-003, TC-027, TC-029 | correct-requirement-no-evidence |

## Closure matrix

| Residual | State | Evidence |
|---|---|---|
| FND-1807 historical exemption population | implemented | Observed classifier results equal the intended two-tree Markdown population |
| FND-1809 copied expected needle list | implemented | Unsealed declaration cross-check replaces the adjacent duplicate and the independence claim |
| Fixture Git census | implemented | Production enumeration sees preferred makefile/workflow and fails on a non-repository |
| FR-006-AC-7 wording | implemented | Requirement, matrix, and declaration state the same repository/census boundary |
| FND-1712 measurement transfer | implemented | SR-010 names both heads and the byte-identical construction |
| FND-1713 dead assertion | implemented | Ownership-first check remains; dead post-creation check is removed |
| Existing real-store leaf | implemented | Canonicalized when present, lexical fallback only on `NotFound` |
| M23 plain target name | implemented | Phony-only preferred makefile expects `compat-view` |
| FND-1810 / Make qualification | deferred | `tl-rewrite#11` and `engineering-assurance#11`; no local parser added |

## Remaining gap

The Make execution-control class is real and remains open. This change improves
the removal census's own retained controls; it does not claim Make execution is
qualified, and it does not replace the common solution with local tooling.

The implementation author recorded local verification at candidate `d4670c25`: the full gate passed, the three census
mutations fail at the intended controls, and the isolation probe passes with an
existing real-store leaf. These observations are not independent clearance. Hosted CI remains manual-only and was not dispatched.
