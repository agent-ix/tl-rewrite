---
id: Task-002
title: "Versioned rule catalog"
type: Task
status: done
track: Core
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/PLAN-001
    type: part_of
---
# Task-002: Versioned rule catalog

## Scope

Define the ordered v1 rewrite catalog with stable rule identity, revision,
profile, precondition, disposition, and semantic provenance.

## Completion Evidence

Tests enumerate enabled and excluded rules, bind implementations to catalog
revisions, and detect revision or priority drift in the catalog digest.
