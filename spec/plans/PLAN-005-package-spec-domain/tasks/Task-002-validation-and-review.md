---
id: Task-002
title: "Validate and review the candidate"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/NFR-003
    type: references
  - target: ix://agent-ix/tl-rewrite/TC-039
    type: verifies
---
# Task-002: Validate and review the candidate

## Scope

Falsify each load-bearing TC-039 control, reconcile exact coverage counts, run
the complete isolated local gate with the released shared-assurance toolchain,
and write closing authorial code and gap reviews.

## Subtasks

- [x] Run temporary mutation probes for every admitted/rejected package-spec
  family, inert metadata/comments, non-literal arguments, triggers, and runtime
  version.
- [x] Run strict Quire validation and coverage plus the complete `make ci` gate
  with Quire 0.31.0, Quoin 0.23.1, and `@agent-ix/ix-flow@0.0.4`.
- [x] Record code-review and gap-analysis artifacts, fix their actionable
  findings, and update the plan completion record.
- [x] Push an exact head and request independent review without dispatching
  hosted CI.

## Deliverables

- Green exact-toolchain local gate and mutation-falsification record.
- Closing SpecReview artifacts and completed plan bundle.
- Exact-head pull request ready for independent merge clearance.

## Notes

- Authorial review is not independent approval.
- Human source-release acceptance remains pending and outside this task.

## Completion Record

Completed on 2026-09-09. Three detached source mutations made TC-039 red for
metadata intrusion, omitted GitHub-shorthand recognition, and disabled
non-literal refusal. Review-time controls additionally exposed and closed npm
global-option and process-substitution gaps. The complete branch-local gate at
`6f13a58` passed 94/94 Quire rows, 106/106 strict documents, all Rust targets,
18/18 shared-assurance tests, MSRV, rustdoc, Cargo Deny, pins, provenance, and
the full Quoin chain. SR-044 and SR-045 record closing authorial review. Hosted
CI was not dispatched; the pushed exact head still requires independent review.
