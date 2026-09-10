---
id: Task-002
title: "Validate and review the candidate"
type: Task
status: not_started
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

- [ ] Run temporary mutation probes for every admitted/rejected package-spec
  family, inert metadata/comments, non-literal arguments, triggers, and runtime
  version.
- [ ] Run strict Quire validation and coverage plus the complete `make ci` gate
  with Quire 0.31.0, Quoin 0.23.1, and `@agent-ix/ix-flow@0.0.4`.
- [ ] Record code-review and gap-analysis artifacts, fix their actionable
  findings, and update the plan completion record.
- [ ] Push an exact head and request independent review without dispatching
  hosted CI.

## Deliverables

- Green exact-toolchain local gate and mutation-falsification record.
- Closing SpecReview artifacts and completed plan bundle.
- Exact-head pull request ready for independent merge clearance.

## Notes

- Authorial review is not independent approval.
- Human source-release acceptance remains pending and outside this task.

