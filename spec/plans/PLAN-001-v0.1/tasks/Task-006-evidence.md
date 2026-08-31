---
id: Task-006
title: "Exact-candidate evidence"
type: Task
status: not_started
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

A passing post-seal collection summary, two passing sealed PGM-01 validations,
and a checksum manifest verify every artifact for the exact clean candidate.
The envelope itself remains non-self-attesting; the post-seal summary records
the external validation result.
