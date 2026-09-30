# Rewrite Criterion workloads

`rewrite_rules` calls the public `rewrite` API on three deterministic
closed-trace graphs. A chain of 1, 24, or 64 `true` conjunctions yields the
same number of real `bool.and.true-left` applications. The caller's node and
application budgets are set exactly to the input size and expected number of
applications.

Preflight and postflight checks assert normalized status, one applied rule per conjunction, and the
one-node output. The timed closure calls the public rewrite entry point and
consumes its report without assertions or report inspection.

The bench uses 20 samples, 500 ms warmup, and 1 s minimum
measurement time. A successful `cargo bench --locked --bench rewrite_rules --
--test` checks all inputs and public outcomes without treating timing as a
regression verdict. Paired baseline/current distributions are evaluated by
the V9 campaign report; this standalone check is not a performance verdict,
and a missing baseline remains incomplete.
