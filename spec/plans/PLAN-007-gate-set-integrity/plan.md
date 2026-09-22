---
id: PLAN-007
title: "Bind the declared and executed CI gate set"
type: Plan
status: done
relationships:
  - target: ix://agent-ix/tl-rewrite/NFR-004
    type: references
---
# PLAN-007: Bind the declared and executed CI gate set

## Objective

Remediate Linear TL-64 / GitHub agent-ix/tl-rewrite#11 by implementing NFR-004:
a dedicated Rust CI entry point, invoked in place of a bare `make ci`, that
(1) refuses to run when the Makefile text or invocation environment carries a
state capable of suppressing prerequisite-failure propagation, and (2)
independently reconciles the set of gates that actually completed against the
declared `ci` prerequisite set, so a false pass on any of the 13 gates is
caught regardless of the mechanism that produced it. No other requirement's
gate logic changes; this plan only adds the binding around it.

**Status (2026-09-22): done.** All 7 tasks complete. Every NFR-004-AC is
implemented and green under 30 library unit tests plus 5 process-level tests
that spawn the real compiled binary, including a direct reproduction of the
tracked TL-64 measurement and a clean positive control. Task-006's real-HEAD
requirement ("confirm it matches a bare `make ci`'s result... both pass, or
both fail for the same reason") is satisfied: run for real against this
repository's actual HEAD, the entry point fails at the same gate a bare
`make ci` fails at, for pre-existing, environment-specific reasons confirmed
via `git stash -u` to already exist on unmodified main (ix-flow version
drift local vs. the `0.0.4` pin; a `quire` column-header assertion mismatch
in `spec/test-matrix.md`) — accepted by the coordinator as sufficient
evidence for this check. Task-007's formal review pass ran:
`spec/reviews/SR-077-tl-64-gate-set-code-review.md` (rust-review; 3
medium/low findings, all fixed — a `Violation.kind` string→enum conversion,
missing test-trace comments, a missing `deny_unknown_fields`) and
`spec/reviews/SR-078-tl-64-gate-set-gap-analysis.md` (no unbacked Test
Matrix row, no unowned code; CONDITIONAL on 3 procedural, non-functional
notes). See log.md for the full closing record and individual task files'
`status`.

## Requirements Summary

### Non-Functional Requirements

- [x] **NFR-004-AC-1**: Static Makefile-text inspection refuses to proceed on
  `SHELL`/`.SHELLFLAGS`/`MAKEFLAGS` assignment, `.ONESHELL:`/`.DEFAULT:`/
  `.IGNORE:`/`.SILENT:`, a `-`-prefixed recipe line, `|| true`, a
  stderr-to-`/dev/null` redirect, `$(eval`, or any of the above inside a
  recursively scanned `include`d file.
- [x] **NFR-004-AC-2**: The entry point does not forward inherited
  `MAKEFLAGS` and invokes Make with an explicit, minimal flag set; an
  `-i`/`-k`/`-S`-equivalent `MAKEFLAGS` in the calling environment is refused
  before Make runs.
- [x] **NFR-004-AC-3**: Each `ci` prerequisite's recipe writes a per-gate
  completion record naming itself and its own recipe's exit status, only on
  that recipe's own successful completion.
- [x] **NFR-004-AC-4**: After Make returns, the entry point reconciles the
  completion-record set against the declared `ci` prerequisite set and
  reports every mismatch in either direction.
- [x] **NFR-004-AC-5**: A missing, removed, or backdated completion record is
  treated as no record, not a pass.
- [x] **NFR-004-AC-6**: The tracked measurement (`.IGNORE:`-prepended
  Makefile copy; skeleton with every recipe replaced by a failing stub) is
  rejected by the entry point.
- [x] **NFR-004-AC-7**: An unmodified Makefile, clean environment, and
  genuinely passing gates yield a zero exit with no violation.
- [x] **NFR-004-AC-8**: README, `CLAUDE.md`, and any hosted workflow dispatch
  reference the entry point rather than a bare `make ci`.

## Dependency Graph

### Core dependency edges
- `Task-001 -> Task-002, Task-003`
  Reason: both the environment-control check and the completion-record work
  build on the entry-point scaffold and the completion-record file contract
  Task-001 defines; neither needs the other to start.
- `Task-001 + Task-003 -> Task-004`
  Reason: reconciliation (AC-4/AC-5) needs both the entry point to reconcile
  from and real completion records to reconcile against.
- `Task-001 -> Task-005`
  Reason: AC-8's documentation/workflow references need the entry point to
  name; content and behavior don't depend on Task-002/003/004.
- `Task-002 + Task-004 + Task-005 -> Task-006`
  Reason: the integration gate exercises every mechanism together
  (static+env pre-flight, reconciliation, and the documented invocation
  path) and cannot pass partially.
- `Task-006 -> Task-007`
  Reason: review and matrix closure happen against a working, gate-passed
  implementation, not against work still in flight.

### Shared dependencies
- The completion-record file format (path convention, gate-name field,
  exit-status field) is a shared contract between the writer (Task-003, the
  13 Makefile recipes) and the reader (Task-004, the entry point's
  reconciliation). Task-001 fixes this contract in its deliverable so
  Task-002/003 can proceed in parallel against a stable shape instead of one
  blocking the other.

### Cross-cutting constraints
- `NFR-003` applies to every gate whose result the Quoin-bound
  `assurance-inputs` chain already reads (`conformance`, `counterexamples`,
  `normalization`, `check-corpus`'s provenance half); this plan does not
  duplicate that coverage and Task-004's reconciliation treats those gates
  identically to the eight NFR-003 cannot see — the completion record, not
  the chain, is what NFR-004 reconciles against.
- Owner-directive constraint (Linear TL-64): all new production logic in
  this plan is Rust. No task introduces a new Python/shell evidence
  framework; `scripts/*.py` stays as-is and out of this plan's scope.

## Test Plan

### Unit Tests
- [x] **TC-057** (NFR-004-AC-1): Static inspector rejects each of `SHELL`,
  `.SHELLFLAGS`, `MAKEFLAGS`, `.ONESHELL:`, `.DEFAULT:`, `.IGNORE:`,
  `.SILENT:`, a `-`-prefixed recipe line, `|| true`, a stderr-to-`/dev/null`
  redirect, and `$(eval` individually, each as its own case; a control
  Makefile with none of the eleven present is accepted. A twelfth case
  places one of the eleven inside an `include`d file and requires rejection
  via the recursive scan.
- [x] **TC-058** (NFR-004-AC-2): Entry point invoked with `MAKEFLAGS` set to
  an `-i`/`-k`/`-S`-equivalent value in the environment is refused before
  Make starts; invoked with a clean environment, it is not.

### Integration Tests
- [x] **TC-059** (NFR-004-AC-3): Running each of the 13 `ci` prerequisites
  individually against its real recipe writes exactly one completion record
  naming that gate and its exit status; a recipe forced to fail (stubbed
  tool) writes no record for that gate.
- [x] **TC-060** (NFR-004-AC-4): Given a completion-record set that is
  missing one declared gate and carries one record for an undeclared gate,
  the entry point reports both mismatches by name; given an exact match, it
  reports none.
- [x] **TC-061** (NFR-004-AC-5): A completion record deleted after being
  written, and one rewritten with a timestamp/revision preceding the
  evaluated run, are each treated as absent by reconciliation.
- [x] **TC-062** (NFR-004-AC-6): The entry point run against a
  `.IGNORE:`-prepended copy of the real Makefile, and against a skeleton
  Makefile with every recipe replaced by a failing stub, each exits non-zero
  with a named violation — the direct reproduction of the tracked
  measurement in TL-64/`agent-ix/tl-rewrite#11`.
- [x] **TC-063** (NFR-004-AC-7): The entry point run against the real,
  unmodified Makefile with a clean environment and every real gate passing
  exits zero with no violation reported.

### Verification (NFRs)
- [x] **verify_documented_paths** (NFR-004-AC-8): Inspect README, `CLAUDE.md`,
  and `.github/workflows/*.yml` for every reference to running the full
  local gate set; each must name the entry point, not a bare `make ci`.
  Inspection, not a test — see SR-073/FND-002.

## Remaining Work

### Remaining Dependency Graph

```
Task-001 (entry-point scaffold + static inspection, AC-1)
   |-- Task-002 (env control, AC-2)              [Track B]
   |-- Task-003 (13 completion records, AC-3)     [Track B]
   |         \
   |          Task-004 (reconciliation, AC-4/5)   [Track A]
   |-- Task-005 (docs/workflow wiring, AC-8)      [Track C]
   \
    (Task-002, Task-004, Task-005) -> Task-006 (integration gate, AC-6/7)
                                          -> Task-007 (review + matrix close)
```

### Track A: Critical Path (serial)

#### A1: Task-001 — entry-point scaffold + static Makefile inspection
- **Scope:** New Rust binary (or example, matching `examples/rule_conformance.rs`
  et al.) that parses CLI args for a target (`ci` initially), parses the
  Makefile text (and any `include`d file, recursively) for the eleven
  execution-control surfaces in AC-1, and refuses with a named violation if
  any is present. Also fixes the completion-record file contract (path,
  gate-name field, exit-status field) as a documented deliverable, not just
  code, so Task-003/Task-004 can build against it independently.
- **Difficulty:** Medium (Makefile text is regular enough for a line/token
  scan; the `include` recursion and `$(eval` ban need care but are still
  textual, matching the deliberately conservative design in NFR-004's Scope).
- **Estimated new code:** ~250-350 lines Rust + fixtures.
- **Exit criteria:** TC-057 green; the record-contract note exists and is
  referenced by Task-003 and Task-004.

#### A2: Task-004 — declared/executed reconciliation
- **Scope:** After the entry point invokes Make (once Task-002's
  environment control and Task-003's completion records exist), read the
  completion-record set, compare it to the declared `ci:` prerequisite list
  parsed from the Makefile, and report every mismatch by name. Treat a
  missing/backdated record as absent (AC-5).
- **Difficulty:** Medium — the comparison itself is a set-difference; the
  care is in record-freshness (AC-5) and in parsing the declared list once,
  the same way Task-001 already parses the Makefile for AC-1.
- **Estimated new code:** ~150-200 lines Rust.
- **Exit criteria:** TC-060, TC-061 green.

#### Gate: Task-006 — integration
- **Measures:** The assembled entry point (Task-001 static check + Task-002
  environment check + Task-003 real completion records + Task-004
  reconciliation) against the tracked reproduction and the clean-pass
  control, run together rather than unit-by-unit.
- **Pass criteria:** TC-062 and TC-063 both green; a real `make ci` run
  through the entry point on the unmodified repository at HEAD passes with
  no violation.
- **If fails:** Do not proceed to Task-007. Bisect which of Task-001/002/003/
  004 the failure traces to before re-attempting the gate — a partial pass
  here is not evidence any single AC is done, since the tracked measurement
  is specifically about mechanisms interacting.

#### A3: Task-007 — review and matrix closure
- **Scope:** Run this repository's spec-driven review step
  (`code-review`/`gap-analysis` equivalent) against the finished
  implementation, fix findings, re-run the Task-006 gate to confirm still
  green, then add TC-057 through TC-063 to `spec/test-matrix.md` marked
  implemented (matching how PLAN-006 closed out TC-041..045) and mark this
  plan's NFR-004 checklist complete.
- **Difficulty:** Easy-Medium, contingent on review findings.
- **Exit criteria:** `make ci` (via the new entry point) exits 0 at the
  merge revision; `spec/test-matrix.md` carries the new rows; NFR-004's
  Acceptance Criteria checklist above is fully checked.

### Track B: Parallel (independent agent, can start once Task-001 lands)

#### B1: Task-002 — invocation-environment control
- **Scope:** Add the `MAKEFLAGS`-sanitizing/explicit-flag-set invocation
  logic to the entry point built in Task-001.
- **Difficulty:** Medium — least-precedented mechanism in this plan (no
  existing repository code does anything like it); flagged by SR-075/FND-003
  as the most likely to need iteration once tested against real `make`
  behavior. Isolated in its own task for exactly that reason.
- **Estimated new code:** ~80-120 lines Rust.
- **Exit criteria:** TC-058 green.

#### B2: Task-003 — per-gate completion records
- **Scope:** Touch all 13 `ci` prerequisite recipes in the Makefile so each
  writes a completion record, matching Task-001's contract, only on its own
  successful completion. SR-075/FND-001: use one shared, mechanical pattern
  (a Make function or variable every recipe calls) rather than 13
  independent hand-edits, to keep the diff reviewable and avoid accidentally
  changing a gate's actual behavior while only intending to add a record.
- **Difficulty:** Medium — mechanically simple per recipe, but widest blast
  radius in this plan (touches recipes nominally owned by FR-001 through
  FR-010 and NFR-001 through NFR-003). Correctness of each gate's own recipe
  is explicitly out of NFR-004's ownership (see NFR-004 Scope); this task
  only adds a record write, and must not change any recipe's actual command.
- **Estimated new code:** ~13 one-line recipe additions + one shared Make
  function/variable definition.
- **Exit criteria:** TC-059 green.

### Track C: Post-Task-001, independent of Track B

#### C1: Task-005 — documentation and workflow wiring
- **Scope:** Update README, `CLAUDE.md`, and any hosted workflow dispatch
  step that currently names `make ci` to name the entry point instead.
- **Difficulty:** Easy.
- **Exit criteria:** `verify_documented_paths` inspection passes (AC-8).

## Parallel Execution Summary

```
Task-001 ─┬─ Task-002 ─────────┐
          ├─ Task-003 ─ Task-004┤
          └─ Task-005 ──────────┴─ Task-006 (gate) ─ Task-007
```
Task-002, Task-003, and Task-005 can run on independent agents once
Task-001 merges; Task-004 needs Task-003's real completion records to test
against, so it starts after Task-003 (not purely parallel to it) even though
neither blocks Task-002 or Task-005.

## Task File Mapping

| Task | Track | Requirement(s) | Depends on | Status |
|---|---|---|---|---|
| Task-001 | A | NFR-004-AC-1 | — | not_started |
| Task-002 | B | NFR-004-AC-2 | Task-001 | not_started |
| Task-003 | B | NFR-004-AC-3 | Task-001 | not_started |
| Task-004 | A | NFR-004-AC-4, AC-5 | Task-001, Task-003 | not_started |
| Task-005 | C | NFR-004-AC-8 | Task-001 | not_started |
| Task-006 | A (Gate) | NFR-004-AC-6, AC-7 | Task-002, Task-004, Task-005 | not_started |
| Task-007 | A | NFR-004 (closure) | Task-006 | not_started |

## Coordination Rules

- Task-001 is a hard single-writer prerequisite for everything else in this
  plan: no other task starts until Task-001's completion-record contract is
  fixed, to avoid Task-003 and Task-004 diverging on record shape.
- `Makefile` is shared mutable state between Task-003 (adds record writes to
  all 13 recipes) and Task-005 (edits header/doc references) and, upstream,
  every other requirement's own gate work. Task-003 and Task-005 should not
  run as literally concurrent edits to `Makefile`/`README.md`/`CLAUDE.md`
  without rebasing serially; they are marked as different tracks for
  *review* independence, not for simultaneous unreviewed edits to the same
  files.
- Do not start Task-006 until Task-002, Task-004, and Task-005 are each
  individually green — the gate is a joint property, not something any one
  of them can pass alone (this is the point of the tracked measurement:
  mechanisms interacting is exactly what a false pass previously hid).
- No task in this plan touches `scripts/*.py`, `assurance/`, or
  `NFR-003`'s owned gates; if a task's implementation seems to need that,
  stop and re-check against NFR-004's Scope section before proceeding.

