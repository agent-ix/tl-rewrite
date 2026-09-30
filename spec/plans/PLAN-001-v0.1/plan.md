---
id: PLAN-001
title: tl-rewrite v0.1 implementation plan
type: Plan
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-001
    type: references
  - target: ix://agent-ix/tl-rewrite/FR-005
    type: references
---

# tl-rewrite v0.1 implementation plan

## Dependency DAG

```text
PGM-01 + exact tl-syntax and tl-mltl revisions
  -> specification and assurance foundation
  -> versioned rule catalog and derivations
  -> bounded rewrite engine and replay traces
  -> bounded equivalence and WEST corpus binding
  -> review remediation and complete local gates
  -> human v0.1 source-release decision
```

## Task File Mapping

| Task | Scope | Exit evidence |
|---|---|---|
| Task-001 | Specification and assurance foundation | Validated requirements, matrix, reviews, and assurance packet |
| Task-002 | Versioned rule catalog | Metadata, derivation, disposition, and revision-drift tests |
| Task-003 | Bounded engine and replay traces | Stable compaction, budget, fixed-point, trace, and replay tests |
| Task-004 | Bounded equivalence and WEST evidence | Exact dependency/corpus identities and complete bounded comparisons |
| Task-005 | Verification and review remediation | Complete local gate and resolved actionable review findings |
| Task-007 | Human source-release decision | Maintainer review and explicit release decision |

## Exit Criteria

All matrix rows are backed by executable or retained inspection evidence, the
complete CI gate passes, no blocking gap remains, and the release stays open
until a human release owner records the source-release decision.
