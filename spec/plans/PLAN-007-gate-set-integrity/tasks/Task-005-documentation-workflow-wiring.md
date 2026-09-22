---
id: Task-005
title: "Documentation and workflow wiring"
type: Task
status: done
track: C
priority: P1
relationships:
  - target: ix://agent-ix/tl-rewrite/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/NFR-004
    type: references
---
# Task-005: Documentation and workflow wiring

## Scope

Close the bypass SR-074/FND-001 identified: `make ci` remains a real
Makefile target, so any documented or hosted path that still names it
directly gets zero protection from this plan while appearing unaffected.
Update every such reference to name the entry point instead.

## Subtasks

- [ ] Update `README.md`'s command list to reference the entry point for the
  full local gate set, keeping `make ci` documented as the underlying
  Makefile target the entry point wraps (not deleting it — it remains
  directly invocable for local convenience per NFR-004's Scope, just no
  longer the assured path).
- [ ] Update `CLAUDE.md`'s Commands section the same way.
- [ ] Update the Makefile header comment block (the one this ticket already
  quotes at length) to describe the entry point rather than describing the
  gap as unremediated.
- [ ] Check `.github/workflows/*.yml` for any `make ci` invocation and
  repoint it at the entry point (hosted CI remains manual-dispatch-only per
  NFR-003; this task does not change that, only what a dispatch runs).

## Deliverables

- README, CLAUDE.md, and Makefile header updated.
- Any hosted workflow reference updated.

## Notes

- This is an Inspection-verified AC (NFR-004-AC-8), not a Test — see
  SR-073/FND-002. Verification is a direct check of these files' content,
  not a written test case.
- Do not remove `make ci` as a Makefile target. The residual disclosure in
  NFR-004's Scope is explicit: a person who deliberately types `make ci`
  directly still bypasses this control, and that is accepted rather than
  solved by making the target unusable for local iteration.
- Unblocks: Task-006 (integration gate; AC-8's inspection is part of the
  gate's pass criteria).
