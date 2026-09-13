---
id: Task-002
title: "Validate and review the candidate"
type: Task
status: in_progress
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/FR-008
    type: references
  - target: ix://agent-ix/tl-rewrite/TC-045
    type: verifies
---
# Task-002: Validate and review the candidate

## Scope

Run the complete local gate, open the pull request, run PR-time code and gap
reviews, fix their findings, and rerun the gate at the exact pushed head.

## Subtasks

- [ ] Run strict Quire validation and coverage and the complete `make ci` gate.
- [ ] Open the pull request referencing #35.
- [ ] Run `rust-review` and `gap-analysis` and fix every verified finding.
- [ ] Rerun the gate at the exact head and record the result on the PR.

## Deliverables

- Exact-head green gate and a mergeable pull request.

## Notes

- Review records are not committed.
- Authorial review is not independent approval.
