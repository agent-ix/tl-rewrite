# Shared assurance

This directory holds what tl-rewrite *declares*. It holds no evidence, no
manifest, no verdict, and no store.

| File | What it is |
|---|---|
| `pins.json` | The Engineering Assurance release this repository adopts, and the digests of the artifacts it reads from that release. Component versions are deliberately not restated: the packaged compatibility matrix is their authority. |
| `change-assurance.json` | The author's statement about the changes under issues #9 and #13, in the shape Quoin's FR-063 record requires. |

## How the pieces relate

```
make assurance-inputs        the ONLY target that runs a producer
   |
   +-> target/assurance/*    structured results, written by domain tools
          |
          +-> scripts/assurance_chain.py        reads those bytes, seals through quoin
          +-> scripts/check_shared_pins.py      classifies the toolchain through the matrix
```

Two rules make this different from what it replaced.

**The driver never produces.** If an input is absent, the chain says so and
names `make assurance-inputs`. It does not run the producer itself. A driver
that can produce its own inputs can produce a green run out of nothing.

**Every attested result is read from producer bytes.** `derive_result()` reads a
field the producer wrote — row outcomes, `matched`, Quire's own coverage totals,
or cargo's `build-finished` message. Nothing is inferred from an exit code alone,
and nothing is scraped from a transcript.

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
store, audit store, tool lock, evidence schema, or aggregate verdict.

There is also no `evidence/` directory and no compatibility view. Issue #13
deleted 582 files under `evidence/`, their only reader, the fixtures and schemas
that served it, and the chain proof obligation that read it, under the authority
of `agent-ix/engineering-assurance#7`. The records were deleted, not
rewritten, and nothing here claims they still verify. Git history and
pull-request review remain the integrity boundary — which is what
`CONTRIBUTING.md` has always said, and the boundary those bytes are now under.
