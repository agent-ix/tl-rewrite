# Rewrite Criterion workloads

`rewrite_rules` calls the public `rewrite` API on three deterministic
closed-trace graphs. A chain of 1, 24, or 64 `true` conjunctions yields the
same number of real `bool.and.true-left` applications. The caller's node and
application budgets are set exactly to the input size and expected number of
applications. Each generated formula's canonical wire SHA-256 must match
`input-digests.json` before Criterion records a sample.

The bench uses Criterion 0.5.1, 20 samples, 500 ms warmup, and 1 s minimum
measurement time. A successful `cargo bench --locked --bench rewrite_rules --
--test` checks all inputs and public outcomes without treating timing as a
regression verdict. Paired baseline/current distributions are evaluated by
the V9 campaign report; a missing baseline remains incomplete.
