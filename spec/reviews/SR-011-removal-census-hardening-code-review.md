---
id: SR-011
title: Closing review of removal-census hardening
type: SpecReview
analysis: code-review
scope: tests/shared_assurance.rs, assurance/change-assurance.json, spec/requirements/FR-006-shared-assurance-intake.md, spec/test-matrix.md
review_set: all
---

# Closing review of removal-census hardening

## Summary

The independent review of PR #18 at `9560e35` raised three high, two medium
and one low findings. The implementation findings are fixed. The broader Make
execution-control class remains explicitly deferred to issue #11 rather than
being claimed closed by this repository-census test.

## Scope and claim

This review hardens TC-029. It does not reinstate repository-local assurance
tooling, claim that Make is qualified, or close issue #11. Hosted CI remains
manual-dispatch only.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-1101 | high | FIXED. The exact deny predicate is applied only to tracked enumeration. Every untracked-not-ignored path is inserted without a second exemption site, and hostile names constrain the predicate against prefix, suffix and makefile widening. | FR-006-AC-7, TC-029 | implementation-bug-despite-evidence |
| FND-1102 | high | FIXED. Exemptions are anchored repository-relative paths: three exact declaration files, plus `.md` only below `spec/reviews/` or `spec/plans/`. Positive and hostile path fixtures exercise the shared classifier, and the observed exact set must equal the expected three paths. | FR-006-AC-7, TC-029 | implementation-bug-despite-evidence |
| FND-1103 | high | FIXED. One independently checked twelve-name set covers the deleted scripts, targets, compatibility-view spellings and both schema filenames. The production exemption-plus-byte scanner must observe every name after non-UTF-8 content. | FR-006-AC-7, TC-029 | correct-requirement-no-evidence |
| FND-1104 | medium | FIXED. The retained missing-path control drives the production scanner and checks its fail-closed diagnostic. Exact-deny hostile paths, anchored exemptions and every deleted needle have retained controls rather than comments alone. | TC-029 | correct-requirement-no-evidence |
| FND-1105 | medium | DEFERRED in scope and guarded against inversion here. TC-029 pins stable clauses of the measured `.IGNORE:` disclosure, rejects live non-comment declarations of the named simple special controls, and requires one literal `ci` declaration. Full use-specific Make producer/runner qualification remains issue #11 under Engineering Assurance #11. | TC-029, issue #11 | correct-requirement-no-evidence |
| FND-1106 | low | FIXED for the census predicate and retained as a residual for all other changes. Direct negative names make a prefix/suffix widening fail even when the current observed Git population is unchanged. Exact-path equality and independent area equality constrain the current tree, while human review remains necessary for semantic changes that preserve the same population. | TC-029 | correct-requirement-no-evidence |

## Population and residual

At this review head TC-029 expects 98 tracked paths, denies exactly the root
lockfile, two root licence texts and `corpus/west-v1/LICENSE`, and scans 94
tracked paths plus every untracked-not-ignored path. Eleven tracked areas are
required. `tests` contains nine paths; removing it leaves 85, so the separately
derived coarse floor is 86. Area equality catches smaller one-file areas.

Issue #11 remains open. These assertions prevent the specific disclosure
inversion found in review; they are not a substitute for the common
producer/runner qualification work tracked by `agent-ix/engineering-assurance#11`.

## Verification record

Exact-head commands and mutation results are recorded in PR #18 after the
final commit. Hosted CI is not dispatched by this change.
