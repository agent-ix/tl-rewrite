# tl-rewrite

Deterministic semantics-preserving rewrites for Mission-time Linear Temporal Logic.

## Commands

```bash
make fmt              # format with rustfmt
make fmt-check        # verify formatting (CI gate)
make lint             # clippy with -D warnings
make test             # cargo test, after the producers have run
make check-corpus     # re-derive corpus, oracle, and dependency provenance
make conformance      # replay every catalog rule through engine and oracle
make counterexamples  # produce and replay the retained counterexample corpus
make normalization    # sweep determinism, fixed point, replay, and budgets
make deny             # cargo deny check advisories, bans, licenses, sources
make audit-unsafe     # check that every unsafe block has a // SAFETY: comment
make spec             # validate and cover specifications with quire
make msrv             # check all targets and features with Rust 1.75
make rustdoc          # build warning-free public documentation
make assurance-env    # build the pinned shared-assurance interpreter
make assurance-inputs # run the producers and write their structured results
make pins             # classify the toolchain through the shared matrix
make compat-view      # read retained evidence through the shared mapping
make assurance-chain  # seal, retain, and verify through quoin
make assurance        # pins + compat-view + assurance-chain
make ci               # every gate locally; hosted CI is manual-dispatch only
```

## Shared assurance

Since issue #9 this repository has no local evidence framework. Retention,
integrity checking, audit, attestation, change records and receipts are owned by
Quoin; static specification and coverage facts come from a Quire export; the
compatibility matrix and the PGM-01 mapping come from Engineering Assurance.
`assurance/pins.json` records the release and the digests of the artifacts read
from it, and `assurance/README.md` explains how the pieces relate.

`make assurance-inputs` is the only target that runs a producer. Everything
downstream consumes those files and refuses to create them.

**Read `Makefile`'s header before trusting a green `make ci`.** The parse-time
guard that policed Make's own execution controls went with the collector it
protected; `.IGNORE:` still neuters ten of the fourteen `ci` prerequisites and
nothing notices. Tracked as `agent-ix/tl-rewrite#11`.

## Safety scaffolding

Backported from `agent-ix/ecaz`:

- `clippy.toml` pins MSRV to `1.75` and caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs in CI and locally via `make audit-unsafe`. Every `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines, or be listed in `scripts/unsafe_comment_baseline.txt`. Update the baseline with `bash scripts/check_unsafe_comments.sh --update-baseline`.
- `rustfmt.toml` uses only stable 100-character-width settings. CI fails on drift.
- `rust-toolchain.toml` pins to stable + rustfmt + clippy.

## Layout

```
src/                      # catalog, bounded engine, replay, and equivalence APIs
examples/                 # the domain producers: rule conformance, counterexamples, normalization
tests/                    # requirement-tagged integration and property evidence
corpus/west-v1/           # checksum-pinned selected WEST inputs
corpus/rules/             # the rule corpus, bound by digest to the constructed fixtures
corpus/counterexamples/   # the counterevidence corpus and its count oracle
assurance/                # what this repository declares: pins and the change-assurance record
evidence/                 # immutable retained records; the verifier was removed, not the record
schemas/                  # retained evidence schemas, no longer validated against
spec/                     # requirements, assurance, reviews, and typed plan bundles
scripts/                  # provenance, pins, compatibility view, and the assurance chain
```
