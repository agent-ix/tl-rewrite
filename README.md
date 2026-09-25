# tl-rewrite

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/6qsdhSPE)

Deterministic, bounded, semantics-preserving rewrites for Mission-time Linear
Temporal Logic with replayable traces and exhaustive bounded equivalence evidence.

## Build

```bash
make guarded-ci
make spec
```

The library requires Rust 1.98.1 and consumes validated `tl-syntax.formula/v1` and
`tl-syntax.formula/v2` documents pinned to revision
`6e2fc17fcfba60c33ab264772bb25550a9c81853` (v0.4.0 candidate). Its immutable future catalog
enables 38 closed-trace rules with stable identity, revision, profile,
precondition, and derivation metadata. Two
growth-sensitive nested Until/Release transformations from WEST paper Theorem 3
are retained as primary-source catalog entries but deliberately excluded.

The separate `past_catalog` admits `mltl.origin-complete-history/v1`, reuses
only the reviewed profile-independent Boolean rules, and adds exactly two
specification-stated canonical folds: `O[1,1] p` to strong Previous, and the
expanded Boolean dual of Since to primitive Triggered. All other past-time
transformations remain unchanged. Rewrites preserve formula-v2, semantic
profile, caller formula identity, source revision, source spans, and replay
bindings; online, mixed, unknown, and structurally invalid inputs are refused
without a partial output.

`rewrite` applies the immutable catalog in bottom-up first-match order under
explicit iteration, node, application, and deterministic logical-work budgets.
Only a fixed point carries a normalized formula. `replay` detects substituted
inputs, catalog/options, steps, intermediates, or output. `check_equivalence`
enumerates every valuation in a horizon-complete bounded closed-trace domain
and delegates verdicts to pinned `tl-mltl` revision
`29cea002008f4a6855f54b73ecd63c234499fa3c` (v0.4.0 candidate). This future-only conformance API
continues to return a typed non-conclusive refusal for past profiles. Past-fold
tests and `check_past_equivalence` compare two `PastEvaluationContext`s -- a
formula paired with the exact history, anchor, and proposition-map identity it
evaluates against -- through the pinned `tl_mltl::past::evaluate_past`
evaluator over event-position and exact fixed-sample histories. Evaluator
refusal, including a resource limit, remains a typed non-conclusive result and
is never coerced to a verdict.

`rewrite_infinite` applies the separate `mltl.infinite-trace/v1` catalog to
validated unbounded graphs. It remaps same-graph fairness roots, enforces
iteration, node, application, work, and report-byte limits, and exposes an
output only after a fixed point. With the `infinite-trace` feature,
`check_infinite_rewrite` replays the exact rewrite and compares both graphs on
one validated lasso through the pinned tl-mltl provider. It classifies
conflicting observations before evaluation. Provider comparison is
trace-scoped evidence; dev-only `tl-oracle` independently qualifies enabled
rules on complete and partial lassos.

The public subsystem layout is `catalog`, `engine::{future,past}`, `infinite`,
`equivalence`, `report`, and `replay`, with the established root API retained as
compatibility re-exports. Shared engine code owns deterministic traversal and
resource charging; profile modules select disjoint rule domains. Every
successful graph is canonically serialized and re-admitted by the real
tl-syntax strict reader. `report::{read,read_with_context}` and
`ReplayReport::from_json_bytes` provide bounded canonical record admission.

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

Licensed under the MIT license. See [LICENSE](LICENSE). Since tl-mltl 0.2.0
(TL-179) the dependency graph carries no
Quire Observation (or other AGPL-3.0-or-later) component, direct or
transitive: this crate, and every crate it depends on for its production
build, stay entirely independent of the agent-ix/Quire ecosystem, per the
TL-175 architect ruling.
