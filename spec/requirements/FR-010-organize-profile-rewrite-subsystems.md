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
`engine::{future,past}`, `infinite`, `equivalence`, `report`, and `replay`. Existing public
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

The `infinite` arm admits only `mltl.infinite-trace/v1` and
`tl-syntax.formula-unbounded/v1`. It uses a distinct catalog under FR-019 and
never turns on an infinite-trace feature in the tl-mltl bounded evaluator.
Qualification uses the independent `tl-oracle` as a dev dependency.
The default build does not enable tl-mltl's non-default `infinite-trace`
feature. The opt-in `tl-rewrite/infinite-trace` feature enables the pinned
provider for trace-scoped differential conformance; it never supplies rule
soundness authority. A runtime comparison remains a diagnostic rather than
a soundness proof.

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

The 2026-09-21 advance to tl-mltl 0.2.0 (agent-ix/tl-rewrite quire-observation
dev-dependency removal, `agent-ix/tl-rewrite#42`/TL-175/TL-179) is the one
exception to the "only revision fields and digests" claim above: tl-mltl 0.2.0
removed `wire::request` and `wire::report` -- the pre-0.2.0 owner-admission
API, itself coupled to quire-observation -- entirely, relocating that surface
to the quire-observation-coupled bridge crate `quire-mltl`, which a TL-* crate
must not depend on. `check_past_equivalence`'s own parameter and report shape
moved with it. `PastEvaluationContext` (a TL-native formula/history/anchor/
proposition-map bundle this crate now owns) replaces `ValidatedTemporalRequest`;
`PastConformanceReport`'s `original_execution`/`rewritten_execution`/
`original_truth`/`rewritten_truth` fields become `original_verdict`/
`rewritten_verdict: Option<bool>`; and `PastConformanceReason::NonBooleanResult`
is removed, because `tl_mltl::past::evaluate_past` (the TL-179 replacement) has
no intermediate non-final result for the past lane -- what used to reach that
reason now reports `OriginalEvaluatorError`/`RewrittenEvaluatorError`, same as
any other evaluator refusal. FR-010-AC-4's own text is unaffected by this
change: past equivalence still invokes only the pinned tl-mltl owner API,
still detects every wrong fold, and still claims no universal proof.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-010-AC-1 | Every existing future and past rule dispatches only through its selected profile catalog; cross-profile, mixed and unknown inputs never reach a rule body. | Test (TC-053) |
| FR-010-AC-2 | Every successful graph passes the real tl-syntax strict reader and preserves contract/profile/source/proposition identity; unproved past forms remain byte-identical. | Test (TC-053) |
| FR-010-AC-3 | Existing canonical graph, schema, status, diagnostic and work-accounting bytes remain unchanged across the source reorganization and public re-exports; only mandatory dependency-revision fields and report/replay digests derived from them advance. | Test (TC-053) |
| FR-010-AC-4 | Past runtime equivalence invokes only the pinned tl-mltl owner API as a diagnostic, detects every wrong O/H/Y/S/T fold across both clocks and anchors, and claims no universal proof; qualification rule evidence uses `tl-oracle`. | Test (TC-053, TC-066) |
| FR-010-AC-5 | Exact and one-over iteration/node/application/work/report/replay bounds preserve existing typed outcomes and expose no partial graph or report. | Test (TC-053) |
| FR-010-AC-6 | Infinite inputs dispatch only through the infinite catalog, keep their profile/edition/fairness identities, and leave the bounded evaluator feature off in default builds; opt-in conformance remains trace-scoped. | Test (TC-067, TC-069) |

## Dependencies

FR-002 supplies the bounded engine, FR-009 supplies past-profile rules,
tl-syntax FR-014 supplies strict graph admission, and tl-mltl FR-018 supplies
the selected evaluator interface used for bounded equivalence.

## Status

Implemented as the tl-rewrite allocation of PLAN-010 Task-009 under
`tl-syntax#52/#64`, with TC-053 covering the complete acceptance surface.
