---
id: PLAN-006
title: "Prove W/M lowering reaches the rewriter as primitive graphs"
type: Plan
status: in_progress
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-008
    type: references
---
# PLAN-006: Prove W/M lowering reaches the rewriter as primitive graphs

## Objective

Close issue #35 by adding test-only controls that compare rewrite outcomes for
tl-syntax W/M lowered graphs against the same primitive graphs built by hand,
inspect `src/` for any derived-operator branch, and falsify the comparison with
lowering mutants. Production source and public APIs stay unchanged.

## Requirements Summary

### Functional Requirements

- [x] **FR-008-AC-1:** Lowered and direct graphs give equal rewrite and
  conformance reports.
- [x] **FR-008-AC-2:** Budget, profile-refusal, and binding outcomes match.
- [x] **FR-008-AC-3:** clean-ascii/v2 parses match by span-insensitive identity.
- [x] **FR-008-AC-4:** `src/` and production dependencies hold no derived branch.
- [x] **FR-008-AC-5:** Every lowering mutant fails parity.

## Dependency Graph

- `tl-syntax#40 (8dc18ee) + tl-parse#31 (9ca856b) -> Task-001`
  Reason: the controls call the merged lowering and v2 parser.
- `Task-001 -> Task-002`
  Reason: the gate and PR-time reviews assess the finished controls.

### The seams

- `Cargo.toml` `[dev-dependencies]`: `tl-syntax-lowering` and
  `tl-parse-derived` aliases. tl-mltl `4bff387` still pins tl-syntax `26b801d`,
  and `src/equivalence.rs` hands `tl_syntax::Formula` to tl-mltl, so the
  production pin cannot move alone.
- `tests/future_lowering_parity.rs`: builders, the formula v1 wire crossing,
  the parity comparator, the source scanner, and the mutants.

## Test Plan

### Integration Tests

- [x] **TC-041:** 18 cases over W/M, four intervals, operand shapes, negation,
  and both nesting orders.
- [x] **TC-042:** four budget kinds, online-prefix refusal, complete and
  incomplete catalogs.
- [x] **TC-043:** four v2 sources, lowering records, span-free identities.
- [x] **TC-044:** node-kind, rule-family, token, and manifest scan with
  synthetic controls.
- [x] **TC-045:** nine mutants, classified as semantic, shape, profile, or span.

### Verification

- [x] Hand-mutate the direct builder (Or to And) and confirm TC-041, TC-042,
  TC-043, and TC-045 go red.
- [ ] Strict Quire validation and coverage and the complete `make ci` gate.
- [ ] PR-time `rust-review` and `gap-analysis`, with findings fixed.

## Remaining Work

### Track A: Critical Path (serial)

- **A1 = Task-001** Lowering lane and parity controls — done.
- **Gate = Task-002** Gate, PR, review, and fixes — pass: exact-head `make ci`
  exit 0 and no unresolved finding.

## Parallel Execution Summary

```text
Task-001 controls -> Task-002 gate, PR, reviews
```

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
|---|---|---|---|---|
| Task-001 | A | FR-008 | TC-041, TC-042, TC-043, TC-044, TC-045 | done |
| Task-002 | A | FR-008 | TC-041, TC-042, TC-043, TC-044, TC-045 | in_progress |

## Coordination Rules

- Add no W/M node, rule, evaluator branch, Quire language, or FRETish parser.
- Do not touch tl-mltl; its #47 owns the evaluator-side controls.
- Keep hosted CI manual-only.

## Post-Plan Merge Gate

Issue #35 closes only after independent review of the exact pushed head and a
merge through the protected path.
