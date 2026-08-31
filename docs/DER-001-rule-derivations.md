# DER-001 — v1 rule derivations

These derivations use the exact `tl-syntax` operator vocabulary and the
`tl-mltl` semantics pinned by this repository. Intervals are
non-empty inclusive integer ranges because `tl-syntax::Interval` requires
`start <= end`. Boolean operations use strong Kleene values for open prefixes;
the identities are enabled only for `mltl.closed-trace/v1` in this catalog.
Online-prefix execution remains excluded until a dedicated prefix-equivalence
population is retained, even where the same algebra is expected to hold.

## Boolean identities (B01–B24)

`not`, `and`, `or`, `implies`, and `equivalent` are defined pointwise by the
truth operations in `tl-mltl`. Direct truth-table enumeration establishes
constant elimination, idempotence, double-negation elimination, implication as
`not p or q`, reflexive implication/equivalence, and equivalence with a Boolean
constant. No distributive or absorption rule is enabled in v1.

## Negation duals (N01–N04)

Future folds inclusive offsets with `or` and Globally folds them with `and`.
De Morgan duality therefore gives `not F[a,b] p = G[a,b] not p` and
`not G[a,b] p = F[a,b] not p`. The pinned evaluator defines Release exactly as
`not ((not p) U[a,b] (not q))`; involution gives both Until/Release negation
rules. The v1 engine exercises them only for closed traces.

## Temporal identities (T01–T10)

An interval `[0,0]` contains only the current instant, giving the four
singleton eliminations. Folding a non-empty interval over constant true/false
gives the Future/Globally constant rules. In Until, a constant-true left
operand satisfies every prefix before a witness, yielding Future. Applying the
declared Until/Release dual to a constant-false Release left operand yields
Globally.

These identities use the pinned evaluator's complete closed-trace definition.
Missing proposition observations are false. Boolean `true` and `false` are
semantic constants rather than observations and remain time-independent after
trace closure; therefore `F[a,b] true` stays true and `G[a,b] false` stays false
even when the trace is shorter than `a + 1`. This is a declared profile
difference from textbook semantics that require that lower-bound instant to
exist. Until uses the exact left window
`[a,i)` before a witness `i`, including when `a > 0`; Release follows by the
declared dual. The retained exhaustive and property populations include
nonzero lower bounds and nesting.

## Primary-source candidates retained but excluded

Elwing et al., “Mission-time LTL (MLTL) Formula Validation Via Regular
Expressions,” iFM 2023, DOI `10.1007/978-3-031-47705-8_15`, Theorem 3 proves
right-nesting identities for bounded Until and Release. Catalog entries
`west.nested-until-right` and `west.nested-release-right` retain that authority
but remain excluded until a separate interval-decomposition policy and
worst-case growth evidence are approved.
