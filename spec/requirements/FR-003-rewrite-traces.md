---
id: FR-003
title: Emit and verify replayable rewrite traces
type: FR
relationships:
  - target: ix://agent-ix/tl-rewrite/StR-001
    type: implements
---

# FR-003: Emit and verify replayable rewrite traces

## Description

Every rewrite attempt shall emit a versioned report that identifies inputs,
outputs or partial state, catalog, strategy, budgets, engine revision, and each
output-changing rule application.

## Behavior

- Steps record sequence, pass, source location, rule identity and its actual
  catalog revision,
  before/after subtree digests, and a rolling intermediate digest.
- Applications in subtrees discarded by an ancestor rewrite are absent from
  the trace and do not consume the output-changing application budget.
- Formula documents remain separate from explanatory trace metadata.
- Verification replays the exact request and compares the complete report.
- Substitution or omission returns a typed mismatch and never a verified flag.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-003-AC-1 | Every output-changing step has exactly one ordered application record with complete identities and intermediate digests. | Test (TC-009) |
| FR-003-AC-2 | Exact replay succeeds, while changed input, catalog, option, step, intermediate, or output fails verification. | Test (TC-010, TC-011) |
| FR-003-AC-3 | Partial and budget-exhausted attempts retain diagnostics but serialize no successful normalized output. | Test (TC-012) |

## Dependencies

Depends on FR-001 and FR-002.
