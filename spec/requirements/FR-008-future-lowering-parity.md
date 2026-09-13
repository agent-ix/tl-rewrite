---
id: FR-008
title: Rewrite lowered W/M graphs exactly like direct primitive graphs
type: FR
relationships:
  - target: ix://agent-ix/tl-rewrite/StR-002
    type: implements
  - target: ix://agent-ix/tl-rewrite/FR-002
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/FR-004
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-008
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-010
    type: depends_on
  - target: ix://agent-ix/tl-parse/FR-008
    type: depends_on
---

# FR-008: Rewrite lowered W/M graphs exactly like direct primitive graphs

## Description

When a formula graph was produced by the tl-syntax lowering of bounded weak
until `W[a,b](p,q)` or strong release `M[a,b](p,q)`, tl-rewrite shall return
the same rewrite, refusal, resource, and conformance outcomes as for the same
primitive graph built directly, and shall hold no derived-operator branch of
its own.

## Inputs

- Formula graphs from `tl_syntax::FutureLoweringRequest::lower()` at tl-syntax
  `8dc18eec5af227f484170362c9e8894b8531a27d`, where
  `W[a,b](p,q)` lowers to `Or(U[a,b](p,q), G[a,b](p))` and `M[a,b](p,q)` lowers
  to `And(R[a,b](p,q), F[a,b](p))`.
- Graphs from `tl_parse::parse_clean_ascii_v2` at tl-parse
  `9ca856b4c040fc2c3329b6defd26a1c9b57de748`.
- The same graphs built by hand from primitive nodes.

## Behavior

- The engine sees only the 12 canonical tl-syntax node kinds: False, True,
  Proposition, Not, And, Or, Implies, Equivalent, Future, Globally, Until,
  and Release. W and M are not node kinds, rule families, or evaluator branches
  in tl-rewrite, and the crate reads no user-authored Quire or FRETish text.
- A lowered graph and its direct twin give equal complete rewrite reports:
  output graph, status, semantic profile, input/request/output identities,
  iterations, work units, rule applications, and ordered steps.
- Equal outcomes hold for budget exhaustion on each budget kind, for the
  refused online-prefix profile, and for contextual rewrite with a complete
  and with an incomplete signal catalog.
- A graph parsed from clean-ascii/v2 text carries source spans. Its
  span-insensitive identities (FR-002-AC-4) equal those of the direct graph.
- The lowering and v2 parser crates are dev-only dependencies under renamed
  package aliases. A lowered graph reaches the production `tl_syntax` type only
  through the formula v1 wire document, byte for byte. The production tl-syntax
  pin moves to the lowering revision only together with tl-mltl.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-008-AC-1 | For W and M over `[0,0]`, `[0,3]`, `[2,5]`, `[1,4]` and `[u32::MAX,u32::MAX]` intervals, distinct and shared operands, constant operands, negation, and both nesting orders, the lowered graph is identical to the direct primitive graph, and its rewrite report and bounded conformance report are therefore equal to the direct graph's. | Test (TC-041) |
| FR-008-AC-2 | The lowered graph is identical to the direct graph under each exhausted budget kind, the online-prefix profile refusal, and complete and incomplete signal catalogs, and the reports are equal, so profile, resource, and refusal identities are preserved. | Test (TC-042) |
| FR-008-AC-3 | A clean-ascii/v2 parse of W/M text, including a nested W-over-M source, lowers to the direct graph's semantic view, names the expected lowering records, gives span-insensitive rewrite and conformance identities equal to the direct graph's, and an unparenthesized W/M chain does not equal the right-associated direct graph. | Test (TC-043) |
| FR-008-AC-4 | Inspection of `src/` and `examples/` finds only the 12 canonical node kinds, only canonical rule families (confirmed against `catalog()`), and no derived-operator, lowering, Quire-language, FRETish, or text front-end token, and the lowering crates are absent from every production dependency section; the scanner flags each synthetic violation and passes a clean control. | Test (TC-044) |
| FR-008-AC-5 | Lowering mutants that swap Until/Release, Or/And, Globally/Future, or the U/R operands, retarget, widen, narrow, or drop the unary node, change the profile, reorder generated nodes, add a node, or attribute spans each fail parity; semantic mutants reach a bounded counterexample, shape mutants change the input identity, and the span mutant keeps it. | Test (TC-045) |

## Dependencies

Depends on FR-002 and FR-004. Implements tl-rewrite's part of tl-syntax FR-010
(`agent-ix/tl-rewrite#35`), after tl-syntax#40 (lowering) and tl-parse#31
(clean-ascii/v2). The production tl-syntax pin stays at `26b801d` while tl-mltl
consumes that revision.
