---
type: log
title: "PLAN-007 update log"
description: "Implementation and verification record for NFR-004 / TL-64 (agent-ix/tl-rewrite#11)."
---
# PLAN-007 update log

## History

- **2026-09-22 — Plan opened.** Scoped to NFR-004, remediating Linear TL-64 /
  GitHub agent-ix/tl-rewrite#11. Decomposed into 7 tasks across tracks A/B/C
  plus one integration gate (Task-006). NFR-004 was authored and reviewed in
  the same session (`quoin:specify`, then `quoin:spec-review` with review_set
  `all`: SR-069 through SR-076), surfacing and closing three gaps before this
  plan was written — a missing tamper-record Acceptance Criterion, an
  incomplete execution-control surface list (missing SHELL/.SHELLFLAGS/
  MAKEFLAGS/$(eval)/include relative to what the removed guard policed), and
  an undisclosed bypass (nothing stopped a caller from still running `make
  ci` directly). All three are reflected in this plan's task breakdown
  (Task-001's expanded surface list, Task-002 for the environment vector,
  Task-005 for the documented-path closure). No implementation code written
  yet; Task-001 is the next unblocked task.

- **2026-09-22 — Tasks 001–006 implemented.** `src/ci_guard.rs` (checkable
  logic: static Makefile scan with recursive `include`, MAKEFLAGS check,
  completion-record read/write/reset, declared/executed reconciliation with
  run-id freshness) and `src/bin/ci_guard.rs` (the `ci`/`record` CLI). All 13
  `ci` prerequisite recipes in `Makefile` now call `$(CI_GUARD) record
  <gate>` as their last step; a new `guarded-ci` target is the assured entry
  point. README, CLAUDE.md, and `.github/workflows/ci.yml` repointed from
  `make ci` to `make guarded-ci` (AC-8). 30 library unit tests (TC-057,
  TC-058, TC-060, TC-061 and their edge cases) and 5 process-level tests in
  `tests/ci_guard.rs` (TC-059, TC-062, TC-063, plus a stale-record control)
  all pass. `cargo fmt --all -- --check` and `cargo clippy --all-targets
  --all-features -- -D warnings` are clean.

  Real-HEAD dogfooding (Task-006) caught a genuine bug before it shipped:
  running `make guarded-ci` against this repository's actual Makefile first
  flagged `--manifest $(...)` and other `\`-continued flag lines as false
  `dash-prefixed-recipe` violations, because the scanner did not track
  backslash line-continuation — Make's `-`-prefix only suppresses failure at
  the *start* of a fresh recipe command, never on a wrapped continuation
  line. Fixed in `src/ci_guard.rs` with continuation-state tracking across
  `scan_file`'s line loop, plus two regression tests
  (`scan_does_not_flag_a_wrapped_flag_on_a_continuation_line`,
  `scan_still_flags_dash_prefix_on_first_line_of_a_continued_recipe`). This
  is exactly the value Task-006's "run for real against HEAD, not just
  fixtures" requirement exists to provide.

  Ran every `ci` prerequisite individually against real HEAD plus this
  plan's changes: `fmt-check`, `lint`, `check-corpus`, `conformance`,
  `counterexamples`, `normalization`, `deny`, `audit-unsafe`, `msrv`, and
  `rustdoc` all exit 0, and `ci_guard` recorded each correctly. `test`,
  `spec`, and `pins`/`assurance` fail — confirmed via `git stash -u` to fail
  *identically* on clean, unmodified main, so none are a regression from
  this plan:
  - `test`: `tests/shared_assurance.rs` — 9 of 18 cases fail with `quoin
    refused the change-assurance record: cannot read --input -: EAGAIN`,
    reproducing byte-for-byte on stashed main. Traced to this machine's
    globally installed `@agent-ix/ix-flow@0.2.3` vs. the repository's pinned
    `0.0.4` (`make pins` independently reports the same drift: `ix-flow:
    0.2.3 -> unknown`). Matches the documented "multi-machine repo drift" /
    "laptop npm registry" pattern for this developer's setup — not a code
    defect and not this plan's to fix.
  - `spec`: `quire validate` rejects `spec/test-matrix.md`'s `Status` column
    header, asserting `Coverage Status` instead — reproduces identically on
    stashed main; pre-existing, already noted in TL-64's own review
    (SR-069/FND-001) as out of scope.
  - `pins`/`assurance`: same ix-flow drift as `test`.
  - Separately, `tests/equivalence.rs::unsupported_profiles_and_domain_limits_are_nonconclusive`
    stack-overflows when run inside the full parallel `cargo test
    --all-targets --all-features` suite (passes in isolation); reproduces on
    stashed main. Worked around for verification only via
    `RUST_MIN_STACK=16777216` in the shell environment — nothing committed.

  Net: a literal 100%-green `make guarded-ci` was not achievable on this
  machine at this revision, entirely for reasons that predate and are
  independent of this plan. `guarded-ci` itself behaved correctly throughout
  — it recorded every gate that genuinely passed, correctly reported every
  gate downstream of a genuine failure as missing rather than papering over
  it, and never produced a false pass. Task-006's own stated tolerance
  ("both pass, or both fail for the same reason") is met.

- **2026-09-22 — Task-007 partially closed.** Added TC-057 through TC-063 to
  `spec/test-matrix.md`'s Non-Functional Requirement Coverage and Test Case
  Summary tables. Checked off all 8 NFR-004-AC rows and the Test Plan
  checklist in `plan.md`. Held open: a separate formal `code-review`/
  `rust-review`/`gap-analysis` pass has not been run (only self-review plus
  clean clippy/fmt and the dogfooding fix above), and the plan's `status`
  frontmatter stays `active` rather than `done` pending either the
  environment blocker clearing or owner sign-off that the evidence gathered
  is sufficient. Nothing in this plan's diff is committed.

- **2026-09-22 — Coordinator accepted the environment-blocker evidence** for
  Task-006 ("same gate fails for the same reason as bare `make ci`" is
  sufficient; the ix-flow/quire drift is known machine-local noise, not
  something to chase now or to hold the plan open for) and asked for the
  formal review pass to be run before closing, per this repository's
  code→review→done flow.

- **2026-09-22 — Formal review pass run; Task-007 and the plan closed.**
  `code-review` (dispatched to `rust-review` per the language table) on
  `src/ci_guard.rs`, `src/bin/ci_guard.rs`, `tests/ci_guard.rs`, `src/lib.rs`,
  and the `Makefile` diff →
  `spec/reviews/SR-077-tl-64-gate-set-code-review.md`. Three real findings
  against this repository's own documented Rust idioms, all fixed in place:
  - `Violation.kind` was `&'static str`; converted to a proper
    `#[non_exhaustive]` `ViolationKind` enum with a `Display` impl, matching
    `src/report.rs::RecordReadErrorCode`'s "stable class of refusal"
    pattern already established in this crate. All 8 push sites and every
    test assertion updated to compare the enum instead of a string.
  - New tests were missing or inconsistent with this repository's
    `// Trace: TC-NNN, FR/NFR-XXX-AC-N` test-tracing convention (seen
    throughout `tests/catalog.rs`, `tests/replay.rs`); every `#[test]` in
    `src/ci_guard.rs` and `tests/ci_guard.rs` now carries its own tag.
  - `GateRecord` derived `Deserialize` without `#[serde(deny_unknown_fields)]`,
    unlike this repository's other parsed-record structs
    (`examples/rule_conformance.rs`'s `Manifest`/`Case`); added, with brief
    field docs.

  `gap-analysis` →
  `spec/reviews/SR-078-tl-64-gate-set-gap-analysis.md`. No unbacked Test
  Matrix row (TC-057 through TC-063 each traced to 1–17 real tests), no code
  with no owning requirement. Three low/medium procedural findings, none
  functional: Task-007 being `in_progress` while this very review ran
  (resolved by finishing Task-007's checklist immediately after); Task-006's
  real-HEAD evidence predating these code-review fixes (accepted given full
  test-suite re-pass and a post-fix `make fmt-check` smoke check; the
  fixes are narrow, mechanical, and fully covered); `parse_dir_flag`
  (test-support CLI plumbing, not itself an AC) has only indirect coverage.

  Re-ran the gates after the fixes: `cargo fmt --all -- --check`, `cargo
  clippy --all-targets --all-features -- -D warnings`, `cargo test --lib
  ci_guard` (30/30), `cargo test --test ci_guard` (5/5) — all clean. Checked
  off Task-007's remaining subtasks, set `plan.md` `status: done`. Nothing
  in this plan's diff is committed; left for the coordinator's own commit.
