---
id: SR-1470
title: "Independent base review of FR-091, NFR-076 and TM-072 (TL-393, TL-405)"
type: SpecReview
analysis: base
scope: "spec/requirements/FR-091-solver-backed-equivalence.md, spec/requirements/NFR-076-default-dependency-tree-unchanged.md, spec/solver-equivalence-test-matrix.md"
review_set: base
---

# Independent base review: tl-rewrite TL-147-spec

## Summary

Checked against `tl-rewrite/src/equivalence.rs`: `check_equivalence` is closed-trace only, ceilings default to 4 propositions, horizon 8, 1,000,000 traces (about 19 bits, matching TL-405), and the `infinite-trace` feature named in FR-091-AC-5 exists. `quire validate` adds one warning (FR-091 line 25, agentless passive); coverage adds 14 planned rows (exempt); the baseline's pre-existing failure (spec.md `MasterRequirements`, SR-009 table row) is unchanged.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | medium | NFR-076 records a baseline file `tools/default-tree.baseline` compared byte for byte: a recorded-baseline antipattern per the checklist, and a churn source on every dependency bump. A `cargo tree` assertion that no tl-solve package is in the default tree states the same property. | NFR-076 Verification |
| FND-002 | medium | FR-091-AC-4 reports decisions at intervals up to 10,000 and the `mismatch` path re-evaluates both formulas with `evaluate_closed`; the tl-mltl evaluator defaults to 1,000,000 node evaluations, so nested wide windows (for example G[0,10000] F[0,10000]) fail the re-check and become typed failures, not mismatches. State the evaluation limits used. | FR-091 Behavior, AC-4 |
| FND-003 | medium | Pin coupling: tl-rewrite's tl-solve revision must carry the same tl-syntax and tl-mltl revisions as tl-rewrite's own. Today tl-rewrite pins tl-syntax 9607722f and tl-mltl 011a4324, while tl-mltl main names 9f7b1c86; adding the edge forces lock-step bumps in two repositories. FR-091-AC-5 detects it after the fact. | FR-091, ADR-004 |
| FND-004 | low | FR-091-AC-4 packs five behaviours into one row; split, so one failure is reportable. | FR-091-AC-4 |
| FND-005 | low | FR-091 line 25 `shall ... be unchanged` is agentless passive (new warning). | FR-091 |
| FND-006 | low | `equivalent` rests on the encoder and solver and the optional proof; it is reported as evidence next to enumeration, which is the stated intent (enumeration stays default). The report carries `basis`; good. TL-405 criterion "agrees with enumeration where decidable" is FR-091-AC-2, strong. | FR-091 |
| FND-007 | low | Verification cells carry `TC-` ids (checklist). | FR-091, NFR-076 |
