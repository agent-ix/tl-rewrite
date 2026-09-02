# tl-rewrite

Deterministic, bounded, semantics-preserving rewrites for Mission-time Linear
Temporal Logic with replayable traces and exhaustive bounded equivalence evidence.

## Build

```bash
make ci
make spec
```

The library consumes validated `tl-syntax.formula/v1` documents pinned to
revision `740182f13b84858008d6f176f75136737d405c1b`. Its v1 catalog enables 38 closed-trace rules with stable
identity, revision, profile, precondition, and derivation metadata. Two
growth-sensitive nested Until/Release transformations from WEST paper Theorem 3
are retained as primary-source catalog entries but deliberately excluded.

`rewrite` applies the immutable catalog in bottom-up first-match order under
explicit iteration, node, application, and deterministic logical-work budgets.
Only a fixed point carries a normalized formula. `replay` detects substituted
inputs, catalog/options, steps, intermediates, or output. `check_equivalence`
enumerates every valuation in a horizon-complete bounded closed-trace domain
and delegates verdicts to pinned `tl-mltl` revision
`fe1c620d7baa743d9c6b4dda27f40d207721fcc9`.

The supported `mltl.closed-trace/v1` profile uses false padding for missing
proposition observations, while Boolean constants remain time-independent at
every queried instant, including after the last observation. Consequently
`F[a,b] true` is true and `G[a,b] false` is false even on traces shorter than
`a + 1`. This deliberately differs from textbook finite-trace conventions that
require the lower-bound instant to exist; equivalence claims here cover only
the declared horizon-complete domain and profile.

## Corpus and evidence

`corpus/west-v1/` is a checksum-pinned, byte-identical MIT-licensed subset of
canonical WEST commit `21cd99ab…`. Ten selected published formulas exercise
enabled rules and are independently checked with tl-mltl. Every enabled rule
also has its own exhaustive small-domain fixture, bound by SHA-256 to
`corpus/rules/manifest.json`. Agreement is conclusive only for the reported
formula pair and domain; it is not a universal proof schema.

`corpus/counterexamples/` is the counterevidence corpus: deliberately unsound
rewrites that must produce a concrete disagreeing trace, and bounded-domain
cases that must decline for a declared reason. Each counterexample is replayed
against both documents outside the enumeration that found it, so a witness that
does not actually separate the pair is a failure rather than a decorative field.

Retention, integrity checking, audit, attestation and receipts are owned
upstream by Quoin, and the compatibility matrix and PGM-01 mapping by
Engineering Assurance. This repository runs its own producers and reports what
those tools said; it keeps no evidence framework of its own. See
`assurance/README.md`.

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
