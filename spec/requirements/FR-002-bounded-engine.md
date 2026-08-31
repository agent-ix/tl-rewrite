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
- Each pass structurally interns identical kind/span nodes, discards nodes that
  are unreachable from the candidate root, and enforces the node budget on the
  reachable compacted graph.
- Cycle fingerprints cover complete compacted states and repeated states return
  the typed `non_convergent` disposition.
- The semantic profile is copied exactly and cannot be selected by a rule.
- Budget, growth, cycle, or validation failure is non-conclusive and has no
  successful normalized output.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-002-AC-1 | Identical inputs and budgets produce byte-identical output, steps, status, counts, and digests. | Test (TC-005) |
| FR-002-AC-2 | Every configured budget and nondegenerate reachable-graph boundary fails explicitly without a successful normalization claim. | Test (TC-006, TC-007) |
| FR-002-AC-3 | Output preserves the exact semantic profile, forms a valid topological graph, and reaches a fixed point under the same catalog. | Test (TC-008) |

## Dependencies

Depends on FR-001 and the exact tl-syntax Formula contract.
