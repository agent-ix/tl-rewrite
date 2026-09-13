---
id: SR-032
title: "Scope and boundary review of M0 assurance follow-ups"
type: SpecReview
analysis: scope-boundary
scope: "FR-006, NFR-003, TM-001 at 6a653b4"
review_set: all
---

## Summary

The repository owns the Rust test guard, workflow identity census, workflow
token, and local verification. It consumes the released shared-assurance pins
and GitHub Actions runner but does not dispatch hosted CI, change the rewrite
engine, recreate assurance infrastructure, or grant release authority.

## Boundary Allocation

| Requirement | Owner | Class |
| --- | --- | --- |
| FR-006-AC-8 | `tests/shared_assurance.rs` | cross-cutting assurance input safety |
| NFR-003-AC-7 | `.github/workflows/ci.yml` plus Rust census in `tests/shared_assurance.rs` | cross-cutting hosted-tool integrity |

| External dependency | Treatment | Contract |
| --- | --- | --- |
| Released Engineering Assurance/Quire/Quoin/ix-flow set | Guaranteed locally | `assurance/pins.json`, TC-023, TC-039 |
| GitHub hosted runner execution | Not exercised or claimed | workflow remains `workflow_dispatch` only |
| Human exact-head review and release acceptance | External human authority | AP-001 and branch protection |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-3201 | medium | **FIXED:** `.github/workflows/ci.yml` was initially in the requirement prose but outside the sealed subject/source boundary; the declaration now allocates it explicitly. | NFR-003-AC-7, hosted-ci |
