# tl-rewrite

Deterministic semantics-preserving rewrites for Mission-time Linear Temporal Logic.

## Commands

```bash
make fmt              # format with rustfmt
make fmt-check        # verify formatting (CI gate)
make lint             # clippy with -D warnings
make test             # cargo test
make check-corpus     # re-derive dependency provenance
make conformance      # replay every catalog rule through engine and oracle
make counterexamples  # produce and replay the retained counterexample corpus
make normalization    # sweep determinism, fixed point, replay, and budgets
make deny             # cargo deny check advisories, bans, licenses, sources
make audit-unsafe     # check that every unsafe block has a // SAFETY: comment
make spec             # validate and cover specifications with quire
make msrv             # check all targets and features with the MSRV toolchain
make rustdoc          # build warning-free public documentation
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
exit 2 to exit 0. `make
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

- `clippy.toml` caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs in CI and locally via `make audit-unsafe`. Every `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines, or be listed in `scripts/unsafe_comment_baseline.txt`. Update the baseline with `bash scripts/check_unsafe_comments.sh --update-baseline`.
- `rustfmt.toml` uses only stable 100-character-width settings. CI fails on drift.

## Layout

```
src/                      # catalog, bounded engine, replay, and equivalence APIs
examples/                 # the domain producers: rule conformance, counterexamples, normalization
tests/                    # requirement-tagged integration and property evidence
corpus/west-v1/           # selected WEST inputs
corpus/rules/             # the rule corpus
corpus/counterexamples/   # the counterevidence corpus and its count oracle
spec/                     # requirements, reviews, and typed plan bundles
scripts/                  # provenance and unsafe-comment checks
```

