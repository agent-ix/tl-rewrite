---
id: Task-001
title: "Restore tracked inputs across unwind"
type: Task
status: done
track: Assurance
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-006
    type: references
  - target: ix://agent-ix/tl-rewrite/TC-038
    type: verifies
---
# Task-001: Restore tracked inputs across unwind

## Scope

Add one Rust restoration guard around the tracked mirror-probe input. Capture
the exact original bytes before mutation, retain the existing shared-input lock,
restore on normal return and panic unwind, and avoid a second panic if restoration
fails while another panic is already active.

## Tests First

TC-038 must cover normal restoration, a forced missing-program spawn failure,
byte-for-byte comparison with the original tracked input, and a scratch-path
restoration failure whose caught payload remains the deliberately raised
original panic.

## Completion Evidence

The focused control passes, and deleting unwind restoration makes its forced
spawn-failure case red without leaving the tracked input modified.

## Completion Record

Completed on 2026-09-09. The token-taking mirror helper now owns a restoration
guard from before mutation through explicit normal restoration; Drop restores
on a forced child-spawn unwind. A scratch-path failure emits an observable report
without replacing the original panic. Focused TC-038 cases passed with exact
post-unwind byte equality. Mutation falsification remains part of Task-003.
