# Infinite rewrite fuzz target

`infinite_rewrite` admits an infinite formula through the syntax owner's
validated wire and canonical reader, runs the bounded rewrite and replay,
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
