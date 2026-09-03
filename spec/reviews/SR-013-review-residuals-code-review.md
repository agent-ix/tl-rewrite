---
id: SR-013
title: Code review of post-merge residual fixes
type: SpecReview
analysis: code-review
scope: tests/shared_assurance.rs, assurance/change-assurance.json, spec/requirements/FR-006-shared-assurance-intake.md, spec/test-matrix.md
review_set: all
---

# Code review of post-merge residual fixes

## Summary

All repository-scoped issue #19 findings are implemented. The executable and
authorial census controls are separately reviewable without being described as
independent authority, and the retained fixtures exercise the production Git
enumeration. Full Make qualification remains deferred to the common work.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-1301 | medium | FIXED. One `git_files` / `census_paths` path now serves live and fixture enumeration; non-repository invocation fails closed with a named diagnostic. | TC-029 | correct-requirement-no-evidence |
| FND-1302 | medium | FIXED. The fixture's phony-only `.PHONY: compat-view` distinguishes the plain needle from the old `compat-view:` spelling. Hostile template and staged excludes controls demonstrate both isolation steps. | FR-006-AC-7, TC-029 | implementation-bug-despite-evidence |
| FND-1303 | low | FIXED. The observed historical-prose set is compared with the intended `.md` population under exactly `spec/reviews/` and `spec/plans/`. | TC-029 | correct-requirement-no-evidence |
| FND-1304 | low | FIXED. Executable denial and deleted-name sets are compared with the declaration's unsealed `census_controls`; no document calls this independent authority. | TC-029 | wrong-requirement |
| FND-1305 | low | FIXED. The dangling probe removes its dead ownership assertion, names setup failures, and resolves an existing real store leaf before containment comparison. | TC-027, NFR-003-AC-3 | correct-requirement-no-evidence |
| FND-1306 | low | FIXED. SR-010 now states why the measured `ec6668b` isolation construction transfers to reviewed landing head `ee5a329`. | NFR-003-AC-3 | correct-requirement-no-evidence |
| FND-1307 | medium | DEFERRED. The remaining expanded Make graph/command qualification is not implemented here and remains deferred to the common work tracked by #11. | NFR-003 | correct-requirement-no-evidence |

## Verification

Local `make ci CARGO_TARGET_DIR=target/cargo-review` passes with 38 Rust tests
and 68/68 Quire rows backed. The complete shared-assurance binary passes 10/10.
Three retained-control mutations were observed red: narrowed `compat-view:`,
removed empty-template override, and widened historical prose. The passing
isolation probe ran while the real Quoin store leaf existed. Hosted CI was not
dispatched.
