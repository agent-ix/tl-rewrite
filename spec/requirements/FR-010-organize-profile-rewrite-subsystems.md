---
id: FR-010
title: "Organize profile-preserving rewrite subsystems"
type: FR
relationships:
  - target: ix://agent-ix/tl-rewrite/MRS-001
    type: implements
  - target: ix://agent-ix/tl-rewrite/FR-002
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/FR-009
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-014
    type: depends_on
  - target: ix://agent-ix/tl-syntax/IF-004
    type: implements
  - target: ix://agent-ix/tl-syntax/VO-006
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-018
    type: depends_on
---
# FR-010: Organize profile-preserving rewrite subsystems

## Description

When a validated formula is rewritten, tl-rewrite SHALL dispatch through the
exact selected profile catalog and preserve the formula owner's contract,
semantic profile, source context and independently replayable result without
introducing a shared future/past rule authority.

## Architecture

The implementation SHALL be organized as `catalog`,
`engine::{future,past}`, `equivalence`, `report`, and `replay`. Existing public
paths remain compatibility re-exports. The shared engine owns deterministic
topological traversal and resource accounting only. Future and past modules own
their disjoint rule admission/preconditions; neither may match a foreign profile
or silently reuse the other's catalog entry.

A private `engine::boolean` helper implements the identical profile-independent
Boolean algebra only after `future` or `past` has admitted the exact profile. It
is one implementation source for the shared catalog entries and owns no temporal
operator admission or authority.

Future formula-v1 behavior, W/M lowering parity and existing report bytes remain
unchanged. Past formula-v2 behavior preserves
`mltl.origin-complete-history/v1`, O/H/Y/S/T nodes and the exact reviewed past
catalog. Only already proved folds are applied; all other valid shapes remain
unchanged. Future, mixed, weak-previous, unknown-profile and unsupported
equivalence requests yield their typed non-conclusive/refused outcome and no
partial rewritten graph.

Reports and replay retain exact input/output formula owner contracts and
identities, catalog/rule revisions, application trace, source context,
proposition bindings, limits and dependency revisions. Every successful output
is admitted by the real tl-syntax strict reader. Past equivalence uses the exact
tl-mltl owner evaluator interface; it never embeds a second evaluator or treats
bounded evidence as universal proof.

A source-only module move changes no canonical graph, semantic identity,
diagnostic, work charge, report or replay byte. The required dependency advance
changes only the exact syntax/evaluator revision fields and report/replay
digests derived from those fields; it does not silently retain a stale producer
identity. No parser, native-language, Contract-IR vocabulary, production
monitor or evidence framework is added.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-010-AC-1 | Every existing future and past rule dispatches only through its selected profile catalog; cross-profile, mixed and unknown inputs never reach a rule body. | Test (TC-053) |
| FR-010-AC-2 | Every successful graph passes the real tl-syntax strict reader and preserves contract/profile/source/proposition identity; unproved past forms remain byte-identical. | Test (TC-053) |
| FR-010-AC-3 | Existing canonical graph, schema, status, diagnostic and work-accounting bytes remain unchanged across the source reorganization and public re-exports; only mandatory dependency-revision fields and report/replay digests derived from them advance. | Test (TC-053) |
| FR-010-AC-4 | Past equivalence invokes only the pinned tl-mltl owner API, detects every wrong O/H/Y/S/T fold across both clocks and anchors, and claims no universal proof. | Test (TC-053) |
| FR-010-AC-5 | Exact and one-over iteration/node/application/work/report/replay bounds preserve existing typed outcomes and expose no partial graph or report. | Test (TC-053) |

## Dependencies

FR-002 supplies the bounded engine, FR-009 supplies past-profile rules,
tl-syntax FR-014 supplies strict graph admission, and tl-mltl FR-018 supplies
the selected evaluator interface used for bounded equivalence.

## Status

Implemented as the tl-rewrite allocation of PLAN-010 Task-009 under
`tl-syntax#52/#64`, with TC-053 covering the complete acceptance surface.
