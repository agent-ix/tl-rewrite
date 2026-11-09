---
id: SR-231
title: "spec-correctness - cargo-mutants configuration, kill-rate thresholds and coverage reporting (tl-rewrite, TL-231, TL-234)"
type: SpecReview
analysis: spec-correctness
scope: "mutation configuration, thresholds, scripts and Makefile verification targets of tl-rewrite"
review_set: all
---

## Summary

TL-231 and TL-234 add a reproducible cargo-mutants configuration, a kill-rate gate, a diff gate and llvm-cov
line and branch coverage reporting to tl-rewrite, as separate Make targets outside `make ci`. This record is the dated
evidence (2026-11-08 and 2026-11-09); `docs/verification-tooling.md` carries no figures. tl-rewrite's engine core is at 94.41%, below the 95% target, with 10 survivors and no tests added; the crate-wide rate (91.12%) is a core plus shard 0/3 sample, not a full run, and the floors (engine 94, crate 88) are the sampled rate minus a margin.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2311 | medium | Mutation measurements (cargo-mutants 27.0.0, `--no-shuffle`, taken on the tree before the final rebase (pre-rebase)): Engine core (`src/engine/*.rs`): 169 caught, 10 missed, 41 unviable (94.41%). Crate sample (engine plus `--shard 0/3`, 175 mutants): 236 caught, 23 missed, 64 unviable (91.12%). Timeouts count against the rate. | `mutation-thresholds.toml` |
| FND-2312 | low | The engine survivors are three deleted `retained_operands` match arms, `reachable_nodes` (`||` to `&&`), two limit comparisons in `build_pass` and `rewrite_validated_with_builder`, the `formula_nodes` limit field, and one condition in `past::apply_first`. `src/ci_guard.rs` and `src/bin/ci_guard.rs` are excluded (123 mutants). | `docs/verification-tooling.md` |
| FND-2313 | low | The enforced mutation floors are the measured sampled rates minus a margin, not the 95% and 85% targets; groups below target have open survivors, listed below, that are test gaps to kill. | `mutation-thresholds.toml` |
| FND-2314 | low | Every gate was shown able to fail: `scripts/mutation_gate.py` exits 1 with `FAIL` when a floor exceeds the measured rate, on an empty population or a failed baseline, and exits 2 on unreadable input; `make coverage COVERAGE_MIN_LINES=99` exits non-zero. `scripts/run_mutants.py` accepts cargo-mutants exit 0, 2 and 3 so the gate always prints its table. | `scripts/mutation_gate.py` |
| FND-2315 | low | Coverage re-measured from a clean `target/llvm-cov-target` on 2026-11-09, all features, `ci_guard` excluded. Stable `make coverage`: 94.35% lines / 93.00% regions / 90.43% functions. Nightly `make coverage-branch`: 89.13% lines / 61.38% regions / 89.64% functions / 90.11% branches. The two toolchains report different line totals and the nightly region figure is much lower, so `COVERAGE_MIN_LINES` is 85, below the lower line figure; both targets pass at that floor. A reviewer measured lower totals on a fresh clone than these; the difference could not be reproduced here, so the floor sits below both readings that were available. | `Makefile` |

## Execution record

Host: Darwin 25.4.0, arm64 (Apple M5 Pro), heavily loaded (load average 35 to 45). Toolchain: rustc 1.98.1,
nightly rustc 1.100.0-nightly (1303417c4 2026-09-21), cargo-mutants 27.0.0, cargo-llvm-cov 0.8.7, quire 0.38.3
(engine 0.52.1). Mutation order is fixed by `--no-shuffle`; there is no seed in this mode. The per-mutant timeout
is the cargo-mutants automatic value (three times the unmutated baseline, minimum 60 s in this crate's
`.cargo/mutants.toml`). Wall times are not equality inputs. The crate-wide rates are core plus `--shard 0/3` file
samples (cargo-mutants shards by file), not full runs.

## Open survivors and timeouts of the measured populations

Each is a test gap to kill, unless listed as equivalent above.

- `src/catalog.rs:98:5: replace infinite_profiles -> Vec<String> with vec!["xyzzy".into()]`
- `src/catalog.rs:98:5: replace infinite_profiles -> Vec<String> with vec![String::new()]`
- `src/catalog.rs:98:5: replace infinite_profiles -> Vec<String> with vec![]`
- `src/engine/mod.rs:106:9: delete match arm "bool.and.true-left" | "bool.and.true-right" | "bool.and.idempotent" | "bool.or.false-left" | "bool.or.false-right" | "bool.or.idempotent" | "bool.implies.true-left" | "bool.implies.false-right" | "bool.implies.reflexive" | "bool.implies.eliminate" | "bool.equivalent.true-left" | "bool.equivalent.true-right" | "bool.equivalent.false-left" | "bool.equivalent.false-right" | "bool.equivalent.reflexive" | "neg.future.dual" | "neg.globally.dual" | "neg.until.dual" | "neg.release.dual" | "temporal.until.true-left" | "temporal.release.false-left" in retained_operands`
- `src/engine/mod.rs:127:9: delete match arm "bool.not.false" | "bool.not.true" | "bool.not.double" | "temporal.future.singleton" | "temporal.globally.singleton" | "temporal.future.false" | "temporal.future.true" | "temporal.globally.false" | "temporal.globally.true" in retained_operands`
- `src/engine/mod.rs:136:9: delete match arm "temporal.until.singleton" | "temporal.release.singleton" in retained_operands`
- `src/engine/mod.rs:148:33: replace || with && in reachable_nodes`
- `src/engine/mod.rs:435:32: replace > with == in build_pass`
- `src/engine/mod.rs:435:32: replace > with >= in build_pass`
- `src/engine/mod.rs:442:9: delete field formula_nodes from struct SyntaxArtifactLimits expression in build_pass`
- `src/engine/mod.rs:680:30: replace > with == in rewrite_validated_with_builder`
- `src/engine/mod.rs:680:30: replace > with >= in rewrite_validated_with_builder`
- `src/engine/past.rs:52:71: replace && with || in apply_first`
- `src/equivalence.rs:376:36: replace * with + in trace`
- `src/equivalence.rs:376:36: replace * with / in trace`
- `src/equivalence.rs:452:9: replace > with >= in check_equivalence`
- `src/equivalence.rs:474:21: replace > with >= in check_equivalence`
- `src/equivalence.rs:490:21: replace > with >= in check_equivalence`
- `src/equivalence.rs:546:5: replace context_identity -> String with "xyzzy".into()`
- `src/equivalence.rs:546:5: replace context_identity -> String with String::new()`
- `src/equivalence.rs:617:9: replace || with && in check_past_equivalence`
- `src/infinite.rs:23:49: replace * with +`
- `src/infinite.rs:23:56: replace * with +`
