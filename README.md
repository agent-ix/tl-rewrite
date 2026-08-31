# tl-rewrite

Deterministic, bounded, semantics-preserving rewrites for Mission-time Linear
Temporal Logic with replayable traces and exhaustive bounded equivalence evidence.

## Build

```bash
make ci
make spec
```

The library consumes validated `tl-syntax.formula/v1` documents pinned to
revision `5e59a26d…`. Its v1 catalog enables 38 closed-trace rules with stable
identity, revision, profile, precondition, and derivation metadata. Two
growth-sensitive nested Until/Release transformations from WEST paper Theorem 3
are retained as primary-source catalog entries but deliberately excluded.

`rewrite` applies the immutable catalog in bottom-up first-match order under
explicit iteration, node, application, and deterministic logical-work budgets.
Only a fixed point carries a normalized formula. `replay` detects substituted
inputs, catalog/options, steps, intermediates, or output. `check_equivalence`
enumerates every valuation in a horizon-complete bounded closed-trace domain
and delegates verdicts to pinned `tl-mltl` revision `a9b78471…`.

## Corpus and evidence

`corpus/west-v1/` is a checksum-pinned, byte-identical MIT-licensed subset of
canonical WEST commit `21cd99ab…`. Ten selected published formulas exercise
enabled rules and are independently checked with tl-mltl. Every enabled rule
also has its own exhaustive small-domain fixture. Agreement is conclusive only
for the reported formula pair and domain; it is not a universal proof schema.

## Development status

This crate is being developed spec-first. Its public API is not stable yet, and
registry publication is disabled until the v0.1 assurance review is complete.

Agent-assisted contributions are reviewed under the same requirements,
testing, provenance, and human release gates as every other contribution.

This crate does not qualify a production monitor, validate a consuming project,
or make an automated release, accreditation, or certification decision.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your
option.
