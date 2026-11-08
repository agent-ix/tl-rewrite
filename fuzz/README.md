# tl-rewrite fuzz targets

`cargo-fuzz` targets for the rewrite engine. Both compare tl-oracle, the
independent reference semantics, on the formula before and after rewriting.

| Target | Input | Check |
| --- | --- | --- |
| `closed_rewrite` | `arbitrary`-built closed-trace (future) or origin-complete (past) formula DAG and a trace | `rewrite()` either refuses with no output, or the oracle's verdict on the output equals its verdict on the input at every trace position |
| `infinite_rewrite` | formula-unbounded/v1 bytes through the strict reader | budgets hold, replay verifies, and the oracle agrees before and after on a fixed lasso |

`closed_rewrite` builds well-formed formulas directly from the fuzz bytes (every
operand refers to an earlier node), so inputs are not spent on decode failures.
Rewrites run under tight budgets, and formulas whose unshared tree exceeds 4096
nodes are skipped because the oracle consumes owned trees. A refused or
non-conclusive rewrite carries no output and so cannot produce a different
verdict. An oracle refusal on the input is a resource decision and is skipped;
an oracle refusal on the rewritten output is a failure.

## Running

```sh
make fuzz-check   # build every target and audit the fuzz crate (part of make ci)
make fuzz-smoke   # replay each checked-in corpus once, no mutation
cargo +nightly fuzz run closed_rewrite fuzz/corpus/closed_rewrite -- \
  -seed=101 -max_total_time=60 -max_len=256 -rss_limit_mb=8192
```

Pass `-rss_limit_mb=8192`. On macOS libFuzzer counts the address sanitizer's
freed memory toward RSS, so the default 2048 MB limit aborts runs whose live
heap is a few tens of megabytes.

The checked-in `closed_rewrite` corpus is a coverage-minimised (`cargo fuzz
cmin`) set of generated inputs. Crashes found by a campaign must be minimised
and promoted to a regression test; a corpus entry is not a fixture.
Run records are in `spec/reviews/` and `spec/evidence/`.
