---
id: SR-612
title: "Evidence record - tl-rewrite fuzz runs (TL-228, TL-29)"
type: SpecReview
analysis: evidence
scope: "fuzz/, closed_rewrite and infinite_rewrite targets, FR-020-AC-1, FR-020-AC-3, TC-066, TC-070, TC-072"
review_set: subset
---

## Summary

Recorded libFuzzer runs of the targets listed below, three RNG seeds of 60 seconds
each per target (8,817,756 executions in total). No run produced a crash, timeout, out-of-memory or sanitizer finding.
These are bounded observations of finite runs: a run without a crash and a coverage
curve that had not flattened are not evidence of correctness (FR-021-AC-5).

## Verdict

**PASS** for the recorded evidence: every run completed, each new target is shown
able to find a seeded fault, and the fuzz gate (`make fuzz-check`) builds every target.
Targets are discharged only by these recorded runs, not by existing.

## Run record

Source revision 0e5ef107df95, 919cba5fd8a9 (campaign tool state clean for `src/`, `Cargo.toml`, `Cargo.lock`);
tools: rustc 1.100.0-nightly (1303417c4 2026-09-21), cargo-fuzz 0.13.1, host aarch64-apple-darwin; sanitizer: address (cargo-fuzz default); libFuzzer flags
`-max_total_time=60 -max_len=256 -rss_limit_mb=8192`; seeds 101, 102, 103 (distinct, run in that order on one
growing corpus per target, or on a throwaway copy of the checked-in corpus for
targets whose checked-in seeds are tested one-by-one). Time is libFuzzer's
reported run time and excludes build time. Machine-readable record:
`fuzz/evidence/2026-11-08/campaign.json`.

| Target | Seed | Seconds | Executions | Coverage edges | Features | Corpus entries | Findings |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| closed_rewrite | 101 | 61 | 80,411 | 3,458 | 14,275 | 1,074 | none |
| closed_rewrite | 102 | 61 | 84,074 | 3,520 | 17,131 | 1,264 | none |
| closed_rewrite | 103 | 61 | 67,407 | 3,531 | 17,523 | 1,294 | none |
| infinite_rewrite | 101 | 61 | 3,431,389 | 3,664 | 7,258 | 1,909 | none |
| infinite_rewrite | 102 | 61 | 2,753,343 | 3,705 | 7,786 | 1,769 | none |
| infinite_rewrite | 103 | 61 | 2,401,132 | 3,732 | 8,200 | 1,788 | none |

## Seeded-fault check

A target is credited only if it fails on a deliberately introduced fault. Each
fault was applied temporarily to the library source (and reverted), and the
target was run from an empty or seed-only corpus.

| Target | Fault | Result |
| --- | --- | --- |
| closed_rewrite | `bool.not.false` rewrite rule emits `true` instead of `false` (src/engine/boolean.rs, one token) | found after 736 executions from an empty corpus (seed 228, `rewrite changed verdict` assertion, oracle disagreement at position 0) |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-61201 | low | `closed_rewrite` found no verdict difference between the input and rewritten formula in any recorded run, for either the closed-trace future family or the origin-complete past family. | campaign.json |
| FND-61202 | low | `infinite_rewrite` (existing target) ran for the first time on a scratch copy of its seeds: no crash. | campaign.json |
| FND-61203 | medium | The repository had no fuzz gate. `make fuzz-check` (builds every target, audits the fuzz crate with cargo-deny, now in `make ci`) and `make fuzz-smoke` (corpus replay) were added; NCSA, the license of libFuzzer, is allow-listed in `deny.toml`. That file is shared with the main crate's `make deny`, but only `libfuzzer-sys` in the fuzz workspace carries the license, so the main crate's graph is unaffected. | Makefile, deny.toml |
| FND-61204 | low | While developing the target, a generated formula with shared operands expanded to an unshared tree large enough to exceed libFuzzer's RSS limit when converted to the oracle's owned trees. The target now skips inputs whose unshared tree exceeds 4096 nodes and builds oracle trees from the root only. | fuzz/fuzz_targets/closed_rewrite.rs |
| FND-61205 | low | On macOS libFuzzer counts the address sanitizer's freed memory toward RSS, so the default 2048 MB limit is too low; recorded runs pass `-rss_limit_mb=8192`. | fuzz/README |

## Limitations

The runs are three 60-second repetitions, not the 3 x 900 s baseline profile of
FR-021, and carry no feature-identity snapshots, so FR-021-AC-1 to AC-4 are not
claimed. The target compares verdicts on traces of at most six positions and
three propositions under tight rewrite budgets; formulas larger than the
budgets are refused rather than compared. The checked-in `closed_rewrite`
corpus is a coverage-minimised set of generated inputs, not a fixture.

