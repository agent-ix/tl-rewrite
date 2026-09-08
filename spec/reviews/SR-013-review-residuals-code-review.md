---
id: SR-013
title: Authorial implementation record for post-merge residual fixes
type: SpecReview
analysis: code-review
scope: tests/shared_assurance.rs, assurance/change-assurance.json, spec/requirements/FR-006-shared-assurance-intake.md, spec/test-matrix.md
review_set: all
---

# SR-013: Authorial implementation record for post-merge residual fixes

## Summary

The implementation author recorded repository-scoped issue #19 work as
implemented. The executable and
authorial census controls are separately reviewable without being described as
independent authority, and the retained fixtures exercise the production Git
enumeration. Full Make qualification remains deferred to the common work.

## Authorship and authority

This is not an independent code review. It is an authorial implementation and
local-execution record for candidate `d4670c25f3300a4859688e0af94f4fd995ba65af`,
later landed on `main`. It grants no review clearance, approval, or merge
authority; a separately authored review must establish those facts.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-1301 | medium | IMPLEMENTED (authorial record; independent clearance pending). One `git_files` / `census_paths` path now serves live and fixture enumeration; non-repository invocation fails closed with a named diagnostic. | TC-029 | correct-requirement-no-evidence |
| FND-1302 | medium | IMPLEMENTED (authorial record; independent clearance pending). The fixture's phony-only `.PHONY: compat-view` distinguishes the plain needle from the old `compat-view:` spelling. Hostile template and staged excludes controls demonstrate both isolation steps. | FR-006-AC-7, TC-029 | implementation-bug-despite-evidence |
| FND-1303 | low | IMPLEMENTED (authorial record; independent clearance pending). The observed historical-prose set is compared with the intended `.md` population under exactly `spec/reviews/` and `spec/plans/`. | TC-029 | correct-requirement-no-evidence |
| FND-1304 | low | IMPLEMENTED (authorial record; independent clearance pending). Executable denial and deleted-name sets are compared with the declaration's unsealed `census_controls`; no document calls this independent authority. | TC-029 | wrong-requirement |
| FND-1305 | low | IMPLEMENTED (authorial record; independent clearance pending). The dangling probe removes its dead ownership assertion, names setup failures, and resolves an existing real store leaf before containment comparison. | TC-027, NFR-003-AC-3 | correct-requirement-no-evidence |
| FND-1306 | low | IMPLEMENTED (authorial record; independent clearance pending). SR-010 now states why the measured `ec6668b` isolation construction transfers to reviewed landing head `ee5a329`. | NFR-003-AC-3 | correct-requirement-no-evidence |
| FND-1307 | medium | DEFERRED. The remaining expanded Make graph/command qualification is not implemented here and remains deferred to the common work tracked by #11. | NFR-003 | correct-requirement-no-evidence |

## Verification

The implementation author locally executed
`make ci CARGO_TARGET_DIR=target/cargo-review` at candidate `d4670c25` and
recorded 38 Rust tests and 68/68 Quire rows backed. The same author recorded
the complete shared-assurance binary as 10/10, and observed three retained
control mutations red: narrowed `compat-view:`, removed empty-template
override, and widened historical prose. The isolation probe ran while the real
Quoin store leaf existed. These are source-attributed observations, not review
clearance. Hosted CI was not dispatched.
