---
id: Task-002
title: "Producers, adapter, and the shared intake path"
type: Task
status: done
track: Implementation
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/PLAN-002
    type: part_of
  - target: ix://agent-ix/tl-rewrite/FR-006
    type: references
---
# Task-002: Producers, adapter, and the shared intake path

## Scope

Emit every rewrite-domain result in a declared structured format, transcribe it
through Quoin without Quoin or Quire executing a producer, obtain the static
specification facts from a Quire export, and read the retained evidence through
the Engineering Assurance compatibility mapping.

## Completion Evidence

Four producers write structured results: `rule_conformance` replays all 40
catalog rules through the engine and the pinned oracle;
`counterexample_evidence` produces the counterevidence corpus and replays each
witness against both documents outside the enumeration that found it;
`normalization_sweep` checks determinism, fixed point, replay, profile
preservation and fail-closed budgets over 864 generated formulas;
`check_provenance.py` re-derives the corpus digests and the dependency pins.

`scripts/assurance_chain.py` reads those bytes and seals, retains and verifies
through the pinned Quoin CLI. Every attested result is derived from a field the
producer wrote, and the chain gates each proof's declared state as well as the
byte identity of what was retained. `scripts/legacy_evidence_view.py` hands each
retained envelope to `map_pgm01_bytes` and reports the answer as it stands.

`tests/shared_assurance.rs` asserts the producer boundary behaviourally with two
runs: producers replaced by logging stubs with the log required to be empty, and
a control that stubs `quoin` and requires the chain to fail.
