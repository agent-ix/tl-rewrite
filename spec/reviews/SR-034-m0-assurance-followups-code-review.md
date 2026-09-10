---
id: SR-034
title: "Rust code review of M0 assurance follow-ups"
type: SpecReview
analysis: code-review
scope: ".github/workflows/ci.yml and tests/shared_assurance.rs at db2018a"
review_set: subset
relationships:
  - target: ix://agent-ix/tl-rewrite/PLAN-004
    type: reviews
  - target: ix://agent-ix/tl-rewrite/TM-001
    type: references
---

## Summary

Reviewed the tracked-file restoration lifecycle, lock ownership, panic behavior,
workflow comment handling, package-token census, trigger extraction, runtime pin,
and their mutation controls at `db2018a`. No remaining code defect was established.

## Verdict

**PASS** — the changed Rust test harness and manual hosted workflow agree with
FR-006-AC-8 and NFR-003-AC-7. This authorial review records local evidence and
does not grant independent exact-head clearance.

## Review Evidence

- The token-taking mirror helper makes callers retain the shared-input guard,
  captures exact bytes before mutation, restores explicitly on normal return,
  and relies on Drop during a forced missing-program panic.
- A forced missing restoration directory exercises the active-unwind branch,
  records the failure, and preserves the deliberately raised original panic.
- Removing Drop restoration made the exact-byte assertion fail; panicking in
  Drop during unwind aborted the mutant process, demonstrating both controls.
- The workflow census ignores comment-only spellings and rejects unscoped,
  alias-form, unversioned, duplicate, block-style automatic, and inline-map
  automatic-trigger mutations. The local released executable reported `0.0.4`.
- The hosted file still declares only `workflow_dispatch`; no hosted run was
  dispatched.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-3401 | low | No remaining code defect found. Review-time gaps for unversioned package tokens and inline-map triggers were corrected and retained as mutations before this candidate. | TC-038, TC-039, `tests/shared_assurance.rs`, `.github/workflows/ci.yml` |
