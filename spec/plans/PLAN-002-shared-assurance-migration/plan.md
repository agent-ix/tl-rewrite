---
id: PLAN-002
title: "Shared assurance migration"
type: Plan
status: in_progress
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-006
    type: references
---
# PLAN-002: Shared assurance migration

## Objective

Move tl-rewrite from its repository-local evidence framework onto the released
Engineering Assurance, Quire, Quoin and ix-flow contracts, preserving every
rewrite-domain behaviour and every retained evidence byte.

## Approach

The rewrite engine is not the subject of this change. Its catalog, budgets,
interning, compaction, cycle detection, replay traces, bounded equivalence and
WEST corpus are carried across unchanged, and the migration only changes how
their results are declared, transcribed, retained and verified.

Four properties shape the design.

**The driver never produces.** One target, `make assurance-inputs`, runs the
producers. Everything downstream consumes those files, and an absent input is an
error naming that target rather than a step the driver quietly performs itself.

**Every attested result is read from producer bytes.** No verdict is inferred
from an exit code alone or recovered from a transcript. This is the failure that
cost Wave 1 a high finding — a chain that sealed `passed` for every proof without
reading what the producer wrote — and it is designed against here rather than
discovered later. The chain additionally gates every proof's *declared state*,
not merely the byte identity of what was retained.

**A counterexample stays a witness.** `check_equivalence` reports the first
disagreeing trace and both verdicts on it. The migration keeps that whole:
`examples/counterexample_evidence.rs` replays each recorded trace against both
documents outside the enumeration that found it, and the chain refuses a
mismatch row whose witness does not actually separate the pair. A counterexample
that has degraded into a boolean is the specific loss this plan exists to
prevent.

**Bounded evidence stays bounded.** Every equivalence row carries the limitation
naming the enumerated domain, and a chain scenario requires it. Nothing here
promotes bounded agreement into a proof of a rewrite schema or into qualification
of a consuming monitor.

## Scope

In scope: the pin declaration, the change-assurance declaration, the driver, the
compatibility view, the three domain producers and the provenance producer, the
rule and counterexample corpora, the Makefile, the hosted workflow, the
specification, and the deletion of the local evidence framework.

Out of scope: rewrite, replay or equivalence behaviour, the WEST corpus contents,
the tl-mltl and tl-syntax pins beyond correcting a constant that disagreed with
them, and any release or publication decision.

## Landing constraints

- Hosted CI is manual-only and is not dispatched by this change.
- Retained evidence bytes are immutable.
- The old generic path is deleted only after both paths have been run against
  the same candidate revision and the result recorded as observed.
- The `tl-syntax` repin that `tl-parse` performed cannot be performed here while
  `tl-mltl` still pins the old revision; the reason is measured and recorded
  rather than worked around.
