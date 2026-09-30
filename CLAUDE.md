# tl-rewrite

Deterministic semantics-preserving rewrites for Mission-time Linear Temporal Logic.

## Commands

```bash
make fmt              # format with rustfmt
make fmt-check        # verify formatting (CI gate)
make lint             # clippy with -D warnings
make test             # cargo test
make check-corpus     # re-derive corpus, oracle, and dependency provenance
make conformance      # replay every catalog rule through engine and oracle
make counterexamples  # produce and replay the retained counterexample corpus
make normalization    # sweep determinism, fixed point, replay, and budgets
make deny             # cargo deny check advisories, bans, licenses, sources
make audit-unsafe     # check that every unsafe block has a // SAFETY: comment
make spec             # validate and cover specifications with quire
make msrv             # check all targets and features with Rust 1.98.1
make rustdoc          # build warning-free public documentation
make assurance-env    # build the interpreter environment hosted CI still calls
make ci               # every gate locally, unguarded (see Makefile header)
make guarded-ci       # the assured entry point; hosted CI is manual-dispatch only
```

## Specification workflow

All new or changed work must be specified before implementation: use `quoin
write` to obtain the current artifact contracts, then update the relevant
requirements, plans, tasks, matrix rows, and evidence links. Before requesting
review, run `quoin review` over the affected scope and validate with Quire.
Record selected analyses and findings; final Quoin acceptance remains a human
decision and must not be advanced automatically.

## CI entry point

**Run `make guarded-ci`, not a bare `make ci`.** The parse-time guard that
used to police Make's own execution controls went with the collector it
protected, and a bare `make ci` still trusts Make's own execution controls
and exit code exactly as before: a single `.IGNORE:` line still takes it from
exit 2 to exit 0 with all 12 `ci` prerequisites reporting success. `make
guarded-ci` wraps it with a Rust program, external to Make (`src/ci_guard.rs`,
`src/bin/ci_guard.rs`), that refuses to invoke Make at all if the Makefile
text or the invocation environment could suppress that propagation, and
reconciles the declared `ci` prerequisite set against the gates that actually
wrote a completion record, independent of Make's own exit code. See
`spec/requirements/NFR-004-gate-set-integrity.md`. `make ci` remains directly
invocable for local convenience and is not itself the assured gate; tracked
as `agent-ix/tl-rewrite#11`.

## Safety scaffolding

Backported from `agent-ix/ecaz`:

- `clippy.toml` pins MSRV to `1.98.1` and caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs in CI and locally via `make audit-unsafe`. Every `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines, or be listed in `scripts/unsafe_comment_baseline.txt`. Update the baseline with `bash scripts/check_unsafe_comments.sh --update-baseline`.
- `rustfmt.toml` uses only stable 100-character-width settings. CI fails on drift.
- `rust-toolchain.toml` pins Rust 1.98.1 + rustfmt + clippy.

## Layout

```
src/                      # catalog, bounded engine, replay, and equivalence APIs
examples/                 # the domain producers: rule conformance, counterexamples, normalization
tests/                    # requirement-tagged integration and property evidence
corpus/west-v1/           # checksum-pinned selected WEST inputs
corpus/rules/             # the rule corpus, bound by digest to the constructed fixtures
corpus/counterexamples/   # the counterevidence corpus and its count oracle
spec/                     # requirements, assurance, reviews, and typed plan bundles
scripts/                  # provenance and unsafe-comment checks
```

There is no `evidence/` directory and no `schemas/` directory. Issue #13 deleted
582 files of retained evidence, their only reader, the fixtures and schemas that
served them, and the chain proof obligation that read it, under the authority of
`agent-ix/engineering-assurance#7`. The records were deleted, not rewritten,
and nothing in this repository claims they still verify. Git history is the
integrity boundary for the deleted bytes.
