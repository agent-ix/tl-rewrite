---
id: SR-073
title: "evidence review of NFR-004 (TL-64 gate-set-integrity remediation)"
type: SpecReview
analysis: evidence
scope: "spec/requirements/NFR-004-gate-set-integrity.md"
review_set: all
---

## Summary

Checked that every Acceptance Criterion names a verification method that
actually produces evidence distinguishable from a false pass, given this
requirement exists specifically because a prior local gate falsely reported
pass. Focused on whether "Test" claims are backed by a described positive
*and* negative control (the class of mistake NFR-003's own Verification
section is careful about), and whether the one "Inspection" criterion
(AC-8, documentation/workflow references) is appropriately non-test.

## Findings

| ID      | Severity | Summary                          | Refs   |
| ------- | -------- | --------------------------------- | ------ |
| FND-001 | low      | Every "Test"-verified AC (AC-1 through AC-7) has a matching positive or negative control named in the Verification section (unmodified-Makefile pass; `.IGNORE:`-prepended and all-`false`-recipe negatives; backdated/removed-record negative; `MAKEFLAGS`-environment negative). No AC lacks a described control. No action needed. | NFR-004 |
| FND-002 | low      | AC-8 (documentation/workflow references name the entry point, not bare `make ci`) is correctly typed as Inspection rather than Test: it is a static fact about README/CLAUDE.md/workflow text, not a runtime behavior of the entry point itself, and inventing a test for it would just be re-implementing grep as a test with no additional evidentiary value over direct inspection. Consistent with NFR-003-AC-9 (also Inspection) for the same reason. No action needed. | NFR-004 |
| FND-003 | medium   | The Verification section's `MAKEFLAGS`-environment control is new relative to what the ticket's own reproduction measured (which only manipulated Makefile text and tool variables, not the invocation environment). Flagging this as evidence not yet collected anywhere — it is a specified control, not yet a demonstrated one, since no implementation exists. This is expected at the specify stage and is not a defect in the requirement; recorded so `spec-to-plan` allocates a task for exactly this control rather than treating AC-2 as covered by the same test as AC-1. | NFR-004 |
