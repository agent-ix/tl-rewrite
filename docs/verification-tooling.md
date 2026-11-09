# Verification tooling

How this crate's mutation, coverage evidence is produced and gated (TL-231, TL-234). The measured numbers below come from a
single machine (Apple M5 Pro, Darwin 25.4.0, rustc 1.98.1, cargo-mutants 27.0.0, cargo-llvm-cov 0.8.7,
nightly rustc 1.100.0-nightly 2026-09-21 for branch coverage) that was heavily loaded by other jobs.

## Targets

| Target | What it does | Budget |
| --- | --- | --- |
| `make mutants` | Full cargo-mutants run (`--no-shuffle`, so the population and order are fixed), then `scripts/mutation_gate.py` against `mutation-thresholds.toml` | hours; weekly, not part of `make ci` |
| `make mutants-diff` | cargo-mutants on `git diff $(MUTANTS_BASE)...HEAD` only (default `origin/main`); every mutant in the diff must be killed | minutes to an hour, per PR |
| `make coverage` | `cargo llvm-cov` line, function and region coverage on the pinned toolchain with a line floor (`COVERAGE_MIN_LINES`) | about 1 to 5 minutes |
| `make coverage-branch` | the same plus branch coverage (unstable, needs nightly with llvm-tools) | about 1 to 5 minutes |


`CI-plumbing` modules are excluded from every threshold (`.cargo/mutants.toml` `exclude_globs` and
`exclude` in `mutation-thresholds.toml`). Equivalent mutants are listed in `exclude_re` with a one-line
reason. `cargo mutants` tests every mutant against the whole test suite, so a full run costs
(mutants x suite time) divided by `MUTANTS_JOBS`; use `MUTANTS_JOBS` to trade memory for time.

## Mutation thresholds

Kill rate is `caught / (caught + missed + timeout)`; unviable mutants count on neither side. The campaign
targets are 95% for the core and 85% for the whole crate. The values enforced by the gate are the measured
floors (they only ratchet up).

| Group | Files | Enforced floor |
| --- | --- | --- |
| `engine-core` | `["src/engine/*.rs"]` | 94.0% |
| `crate` | `["src/*.rs", "src/*/*.rs"]` | 88.0% |

Measured at 3e20b78 on this machine: core run: 220 mutants (169 caught, 10 missed, 41 unviable = 94.41%). Crate sample: engine plus `--shard 0/3` (175 mutants: `src/catalog.rs`, `src/equivalence.rs`, `src/infinite.rs` and part of `src/engine/mod.rs`) = 236 caught, 23 missed, 64 unviable, 91.12%. `src/ci_guard.rs` and `src/bin/ci_guard.rs` are excluded (112 + 11 mutants).

The crate floor is the sampled rate minus a margin, because the sample covers only the files named
above; the first full `make mutants` run replaces the sample and the floor should then be raised to the
measured value.

The gate is shown to fail: with a floor above the measured rate it prints `FAIL` and exits 1.
Unresolved survivors and timeouts of the measured populations (each is a `test_gap` to kill with a test,
or an equivalent mutant to list in `exclude_re` with a reason):

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

## Coverage

`make coverage-branch` on this tree (all features, `ci_guard` excluded):

| Metric | Value | Metric | Value | Metric | Value | Metric | Value |
| --- | --- | --- | --- | --- | --- | --- | --- |
| lines | 93.14% | regions | 88.09% | functions | 87.57% | branches | 87.70% |

The stable-toolchain `make coverage` reports slightly different totals than the nightly branch run (it counts test sources too), so `COVERAGE_MIN_LINES` is set a point or two below the lower of the two. It is a floor, not a target; it fails the target when line coverage drops below it.
