---
id: FR-002
title: Execute deterministic bounded rewrites
type: FR
relationships:
  - target: ix://agent-ix/tl-rewrite/StR-002
    type: implements
---

# FR-002: Execute deterministic bounded rewrites

## Description

The library shall apply the selected catalog in stable bottom-up first-match
order under explicit iteration, node, rule-application, and logical-work budgets.

## Behavior

- Each pass traverses the validated topological graph in stable node order.
- Each pass structurally interns identical semantic node kinds through a
  deterministic logarithmic-time index. Source spans remain diagnostic
  provenance and do not split semantic interning, formula identities, or cycle
  fingerprints. Each pass discards nodes that are unreachable from the
  candidate root and enforces the node budget on the reachable compacted graph.
- Retained trace provenance is selected by one budgeted reachability walk over
  per-source operand edges; it never materializes transitive contributor sets.
- Cycle fingerprints cover complete compacted states and repeated states return
  the typed `non_convergent` disposition.
- The semantic profile is copied exactly and cannot be selected by a rule.
- Budget, growth, cycle, or validation failure is non-conclusive and has no
  successful normalized output.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-002-AC-1 | Identical inputs and budgets produce byte-identical output, steps, status, counts, and digests. | Test (TC-005) |
| FR-002-AC-2 | Every configured budget, nondegenerate reachable-graph boundary, retained-source traversal boundary, and repeated complete state fails explicitly without a successful normalization claim. | Test (TC-006, TC-007, TC-020, TC-021) |
| FR-002-AC-3 | Output preserves the exact semantic profile, forms a valid topological graph, and reaches a fixed point under the same catalog. | Test (TC-008) |
| FR-002-AC-4 | Inputs that differ only in diagnostic source spans intern and rewrite identically, and their formula-level replay identities are equal. When equivalent nodes merge, their output diagnostic span is the first encountered source span; per-step source spans retain the input location of the applied rule. | Test (TC-040) |

## Dependencies

Depends on FR-001 and the exact tl-syntax Formula contract.
