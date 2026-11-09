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
line and branch coverage reporting to tl-rewrite, as Make targets (only `make mutants-diff` is in the guarded gate). TL-371 (2026-11-09) replaces the sampled floors with the first full run, adds the per-survivor reason check and puts `make mutants-diff` in `make guarded-ci`. This record is the dated evidence; `docs/verification-tooling.md` carries no figures. The engine core is at 100.00% and the crate at 99.76%; floors are 100.0 and 99.7.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2311 | medium | First full `make mutants` (cargo-mutants 27.0.0, `--no-shuffle`, 2026-11-09): 518 mutants, 386 caught, 35 missed, 97 unviable, 0 timeouts. Engine core 94.41% (10 missed); crate 91.69%. Wall time 8026 s. The test-timeout floor was raised from 60 s to 300 s after the parallel tl-parse run counted 277 load-induced timeouts at 60 s (host load average 13 to 19); the tl-rewrite run had none. | `mutation-thresholds.toml` |
| FND-2312 | low | `tests/mutation_survivors.rs` adds behavior tests for the node budget boundary, shared-subgraph compaction, singleton until and release, the strong-previous fold, counterexample trace layout, the record ceilings and context identities, the infinite rewrite (iteration count, report byte ceiling, empty revision, replay) and the past equivalence refusals. Every survivor was retested with the full suite after the tests landed. `src/ci_guard.rs` and `src/bin/ci_guard.rs` are excluded (123 mutants). | `tests/mutation_survivors.rs` |
| FND-2313 | low | Result after the new tests, equivalent mutants excluded as `.cargo/mutants.toml` now does: engine core 100.00% (176 caught, 0 missed, 41 unviable), crate 99.76% (413 caught, 1 missed, 97 unviable). Floors are the measured rates truncated: engine 100.0, crate 99.7. Seven mutants are excluded as equivalent or unreachable, each with a reason in `.cargo/mutants.toml`. | `mutation-thresholds.toml`, `.cargo/mutants.toml` |
| FND-2314 | low | Every gate was shown able to fail: `scripts/mutation_gate.py` exits 1 with `FAIL` when a floor exceeds the measured rate, on an empty population or a failed baseline, and exits 2 on unreadable input; `make coverage COVERAGE_MIN_LINES=99` exits non-zero. `scripts/run_mutants.py` accepts cargo-mutants exit 0, 2 and 3 so the gate always prints its table. | `scripts/mutation_gate.py` |
| FND-2315 | low | Coverage re-measured from a clean `target/llvm-cov-target` on 2026-11-09, all features, `ci_guard` excluded. Stable `make coverage`: 94.35% lines / 93.00% regions / 90.43% functions. Nightly `make coverage-branch`: 89.13% lines / 61.38% regions / 89.64% functions / 90.11% branches. The two toolchains report different line totals and the nightly region figure is much lower, so `COVERAGE_MIN_LINES` is 85, below the lower line figure; both targets pass at that floor. A reviewer measured lower totals on a fresh clone than these; the difference could not be reproduced here, so the floor sits below both readings that were available. | `Makefile` |

## Execution record

Host: Darwin 25.4.0, arm64 (Apple M5 Pro), heavily loaded (load average 35 to 45). Toolchain: rustc 1.98.1,
nightly rustc 1.100.0-nightly (1303417c4 2026-09-21), cargo-mutants 27.0.0, cargo-llvm-cov 0.8.7, quire 0.38.3
(engine 0.52.1). Mutation order is fixed by `--no-shuffle`; there is no seed in this mode. The per-mutant timeout
is the cargo-mutants automatic value (three times the unmutated baseline, minimum 300 s in this crate's
`.cargo/mutants.toml`, raised from 60 s in TL-371). A mutant is timed out when its test run exceeds the larger of 300 s and three times the baseline; the floor cannot tell a hang from a run that a loaded host slowed past it, so timeouts are counted as unkilled. Wall times are not equality inputs. The 2026-11-09 rates are a full run plus retests of every survivor; a mutant a new test kills stays killed, so the merge is the rate a fresh full run would report.

## Survivors

One mutant is neither killed nor equivalent, listed with its reason under `[[survivor]]` in `mutation-thresholds.toml`: the `input_fairness_identity.is_none()` operand of the fairness guard in `rewrite_infinite`, which only fires when a fairness document cannot be hashed. It counts against the rates above. The review-found survivors (the equivalence `trace` bit layout and the owner node-limit refusal in `compact`) are killed by `tests/mutation_survivors.rs`. The Campaign runner cannot contain `make mutants` (TL-28, 2026-09-24); `make mutants` is the supported path.
