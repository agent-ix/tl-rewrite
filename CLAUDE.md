# tl-rewrite

Deterministic semantics-preserving rewrites for Mission-time Linear Temporal Logic.

## Commands

```bash
make fmt            # format with rustfmt
make fmt-check      # verify formatting (CI gate)
make lint           # clippy with -D warnings
make test           # cargo test
make build          # release build
make clean          # cargo clean
make deny           # cargo deny check advisories, licenses, and sources
make audit-unsafe   # check that every unsafe block has a // SAFETY: comment
make check-corpus   # verify retained WEST bytes
make spec           # validate and cover specifications
make evidence-tool  # test evidence behavior and schemas
make verify-evidence # verify retained evidence checksum manifests
make check-tool-identities # host-scoped qualification identity gate
make ci-for-evidence # portable candidate gates plus the host-scoped identity gate
make ci             # portable complete gate, including retained evidence
```

## Safety scaffolding

Backported from `agent-ix/ecaz`:

- `clippy.toml` pins MSRV to `1.75` and caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs in CI and locally via `make audit-unsafe`. Every `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines, or be listed in `scripts/unsafe_comment_baseline.txt`. Update the baseline with `bash scripts/check_unsafe_comments.sh --update-baseline`.
- `rustfmt.toml` uses only stable 100-character-width settings. CI fails on drift.
- `rust-toolchain.toml` pins to stable + rustfmt + clippy.

## Layout

```
src/                    # catalog, bounded engine, replay, and equivalence APIs
tests/                  # requirement-tagged integration and property evidence
corpus/west-v1/         # checksum-pinned selected WEST inputs
spec/                   # requirements, assurance, reviews, and typed plan bundle
scripts/                # safety and evidence tooling
```
