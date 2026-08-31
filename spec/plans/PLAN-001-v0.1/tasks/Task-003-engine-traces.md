---
id: Task-003
title: "Bounded engine and replay traces"
type: Task
status: done
track: Core
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/PLAN-001
    type: part_of
---
# Task-003: Bounded engine and replay traces

## Scope

Implement stable bottom-up rewriting, structural interning, reachable-graph
compaction, cycle detection, output-relevant step recording, and exact replay.

## Completion Evidence

Tests cover deterministic output, every budget, nondegenerate graph limits,
discarded subtrees, fixed points, catalog revisions, rolling digests, and replay
substitution.
