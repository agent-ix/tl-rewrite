# Shared assurance

This directory holds what tl-rewrite *declares*. It holds no evidence, no
manifest, no verdict, and no store.

| File | What it is |
|---|---|
| `pins.json` | The Engineering Assurance release this repository adopts, and the digests of the artifacts it reads from that release. Component versions are deliberately not restated: the packaged compatibility matrix is their authority. |
| `change-assurance.json` | The author's statement about the change under issue #9, in the shape Quoin's FR-063 record requires. |

## How the pieces relate

```
make assurance-inputs        the ONLY target that runs a producer
   |
   +-> target/assurance/*    structured results, written by domain tools
          |
          +-> scripts/assurance_chain.py        reads those bytes, seals through quoin
          +-> scripts/legacy_evidence_view.py   reads evidence/ through the pinned mapping
          +-> scripts/check_shared_pins.py      classifies the toolchain through the matrix
```

Three rules make this different from what it replaced.

**The driver never produces.** If an input is absent, the chain says so and
names `make assurance-inputs`. It does not run the producer itself. A driver
that can produce its own inputs can produce a green run out of nothing.

**Every attested result is read from producer bytes.** `derive_result()` reads a
field the producer wrote — row outcomes, `matched`, Quire's own coverage totals,
or cargo's `build-finished` message. Nothing is inferred from an exit code alone,
and nothing is scraped from a transcript.

**A refusal is a result.** This repository's eleven retained envelopes are
`quire.derivation-evidence/v1`, which the pinned PGM-01 mapping does not cover,
so it answers `incompatible` for every one of them. That answer is reported as it
stands. It is not a pass, it is not a defect of those records, and it is not a
reason to write a local mapper — which is precisely what this migration removed.
Filed upstream as `agent-ix/engineering-assurance#21`.

## What the rewrite domain contributes

Four producers, all owned here:

- `examples/rule_conformance.rs` replays every catalog rule through the real
  engine and the pinned `tl-mltl` oracle.
- `examples/counterexample_evidence.rs` produces the counterevidence corpus and
  **replays each witness** against both documents outside the enumeration that
  found it. A counterexample that has become a boolean is the failure that
  producer exists to prevent.
- `examples/normalization_sweep.rs` sweeps determinism, fixed point, replay,
  profile preservation and fail-closed budgets over a generated family.
- `scripts/check_provenance.py` re-derives the retained corpus digests and
  requires the revision constants this crate publishes as wire fields to be the
  revisions Cargo actually resolved.

Bounded agreement stays bounded. Every equivalence row carries the limitation
naming the enumerated domain, and a chain scenario requires it to be there.

## What is not here

No evidence envelope, manifest, anchor file, retraction registry, retention
store, audit store, tool lock, or aggregate verdict. Retained bytes under
`evidence/` are immutable, and Git history plus pull-request review are the
integrity boundary for them — which is what `CONTRIBUTING.md` has always said.
The compatibility view asks Git rather than implying a stronger claim than it can
make.
