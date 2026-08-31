---
id: Task-006
title: "Exact-candidate evidence"
type: Task
status: done
track: Evidence
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/PLAN-001
    type: part_of
  - target: ix://agent-ix/tl-rewrite/MP-001
    type: references
---
# Task-006: Exact-candidate evidence

## Scope

Retain the exact clean revision's local results, tool and dependency identities,
PGM-01 checks, and explicit limitations in a checksummed evidence record.

## Completion Evidence

The follow-up `9e736b25dcc1` record has a passing post-seal collection summary,
two passing sealed PGM-01 validations, and a checksum manifest that verifies
every artifact. AA-001 anchors that manifest outside the archive, and the gate
rederives the summary from status files. The envelope itself remains
non-self-attesting; the post-seal summary records the external validation result.
The preceding `4b716afa9d0f` record is retained and anchored as failed evidence because its
temporary validator environment was unavailable; it is not presented as passing evidence.
