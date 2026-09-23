# Infinite rewrite fuzz target

`infinite_rewrite` admits raw input through the syntax owner's strict reader,
runs the bounded rewrite and replay,
then compares original and output graphs with the independent dev-only
`tl-oracle` on one partial lasso. The checked corpus seeds exercise a Boolean
fold, an unbounded future fold, and a past operator. Resource refusals remain
non-passing for semantic comparison.

Run a short local campaign with an installed nightly toolchain:

```bash
rustup run nightly cargo fuzz run infinite_rewrite -- -runs=100
```

The target's exact Git dependency on `tl-oracle` is development-only and is
absent from the production `tl-rewrite` crate graph.

The three checked seeds are canonical strict-reader documents. The seed test
checks their digests and that they reach real rewrite rules. To retain a
bounded V4 libFuzzer run on a clean source commit with nightly `cargo`/`rustc`:

```bash
export PATH="$(dirname "$(rustup which --toolchain nightly cargo)"):$HOME/.cargo/bin:$PATH"
PYTHONPATH=fuzz python3 -m unittest fuzz.test_run_v4_campaign
python3 fuzz/run_v4_campaign.py --output fuzz/evidence/v4-2026-09-23 \
  --runs 1000 --seed 181 --seconds 30
```

The runner copies seeds to scratch and retains raw streams, exact source,
toolchain and lock identities, budget, actual executions, stop reason, and
crash artifact hashes. A crash remains incomplete until its input is minimized
and replayed on the same revision. A clean finite run is bounded evidence.
