---
id: Task-002
title: "Invocation-environment control"
type: Task
status: done
track: B
priority: P1
relationships:
  - target: ix://agent-ix/tl-rewrite/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/NFR-004
    type: references
  - target: ix://agent-ix/tl-rewrite/TC-058
    type: verifies
---
# Task-002: Invocation-environment control

## Scope

Close the vector static Makefile-text inspection (Task-001) cannot see: a
caller whose shell environment sets `MAKEFLAGS` to an `-i`/`-k`/`-S`-
equivalent value, without touching the Makefile at all. The entry point must
not forward an inherited `MAKEFLAGS` and must invoke Make with an explicit,
minimal flag set.

## Subtasks

- [ ] Before invoking Make, read the calling environment's `MAKEFLAGS` (if
  set) and parse it for `-i`/`--ignore-errors`, `-k`/`--keep-going`,
  `-S`/`--no-keep-going`-negation, or any other flag equivalent to ignoring
  or continuing past a recipe failure.
- [ ] If any such flag is present, refuse before Make runs, reporting the
  offending flag.
- [ ] Otherwise, invoke Make with an explicit flag set constructed by the
  entry point (not by forwarding the process's inherited environment), so a
  clean environment is not accidentally polluted by unrelated inherited
  flags either.
- [ ] Add a positive control (clean environment) and the negative control
  (sabotaged `MAKEFLAGS`) as fixtures.

## Deliverables

- The environment-control check wired into the entry point built in
  Task-001.
- TC-058 test and fixtures, green.

## Notes

- SR-075/FND-003: this is the least-precedented mechanism in this plan (no
  existing code here does anything like it) and the most likely to need
  iteration once tested against real `make` behavior across platforms — keep
  this task's scope narrow (`MAKEFLAGS` only) rather than trying to
  anticipate every possible environment-based override in the first pass.
- Does not block or get blocked by Task-003; both depend only on Task-001.
- Unblocks: Task-006 (the integration gate exercises this alongside every
  other mechanism).
