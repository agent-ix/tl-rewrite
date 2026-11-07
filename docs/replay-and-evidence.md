# Replay and evidence

A rewrite report states what the engine did. Replay and bounded equivalence are the two checks that the crate runs on that statement.

## Replay

`replay(input, expected_report)` re-runs `rewrite` with the formula identity, options and source revision recorded in `expected_report`, then compares the whole observed report with the expected one. It returns a `ReplayReport` with status `Verified` when they are equal and `Mismatch` when any field differs. The report holds the SHA-256 of both reports so a caller can see which side changed.

Replay detects a substituted input, a different catalog or option set, an altered, added or removed step, a changed intermediate digest and a changed output. A mismatch says only that the report is not the result of this request. It does not say which field was edited.

`replay_with_context` does the same for a contextual report. The caller supplies the complete signal catalog and the exact requirement context again, because both feed the request digest. The result uses the `tl-rewrite.replay/v2` schema and records the catalog digest and the context.

`replay_infinite` re-executes an infinite report under a trusted source revision and returns a Boolean. It returns false for a failed report, a different revision or catalog digest, or any difference in the re-executed result.

## Strict record readers

`RewriteReport::from_json_bytes`, `ReplayReport::from_json_bytes` and `InfiniteRewriteReport::from_json_bytes` read a report from bytes under `RecordLimits`: maximum bytes, maximum JSON depth and maximum string length. The defaults are the owner maxima (64 MiB, depth 128, 64 KiB strings). A caller can lower a limit and cannot raise it. A reader accepts only closed records with a known schema version and only the one canonical encoding, which is what `serde_json::to_vec` produces for the type. It returns a `RecordReadError` with one of four codes: `ResourceLimit`, `Malformed`, `NonCanonical` and `ExpectedMismatch`.

`report::read(bytes, input, limits)` and `report::read_with_context(...)` go one step further. They read the record and then replay it against the supplied input, and return the report only when the replay verifies. A report that parses but does not replay is refused with `ExpectedMismatch`.

## Bounded equivalence

`check_equivalence(original, rewritten, comparison_id, options)` compares two closed-trace formulas on every trace in a complete bounded domain, using tl-mltl to evaluate each side.

1. The horizon is the larger lookahead of the two formulas.
2. Traces have length `horizon + 1`. The domain is every assignment of true or false to every proposition at every instant, so it has `2^(length * propositions)` traces.
3. Both formulas are evaluated on each trace in a stable order. The first trace on which the verdicts differ ends the run.

The result is a `ConformanceReport` with one of three statuses.

| Status | Meaning |
| --- | --- |
| `Equivalent` | Every trace in the domain agreed. The report gives the horizon, the trace length and the number of traces checked. |
| `Mismatch` | The report carries the first disagreeing trace as a list of instants, each a list of the proposition ids that are true then, and the verdict of each formula on it. |
| `NonConclusive` | The comparison did not finish. `reason` is a `ConformanceReason`. |

`ConformanceOptions` sets the ceilings: 4 propositions, a horizon of 8 and 1,000,000 traces by default. A comparison that would exceed one returns `NonConclusive` with `PropositionLimit`, `HorizonLimit` or `TraceDomainLimit`. The other reasons are `InvalidInput`, `UnsupportedProfile`, `EvaluatorError` and, for contextual checks, `OriginalBinding` and `RewrittenBinding`. A non-conclusive result is never reported as agreement or as a verdict.

The report records the catalog digest, the proposition set and a limitation string that restates the boundary: agreement covers traces of exactly `horizon + 1` instants under the closed-trace profile and does not prove a rewrite schema.

`check_equivalence_with_context` adds the signal catalog and requirement context, and refuses with a binding reason when either formula names an unbound proposition.

## Past and infinite comparisons

`check_past_equivalence` compares two `PastEvaluationContext` values. Each pairs a formula with the history, anchor and proposition-map identity it is evaluated against, and the two contexts must share all three. Evaluation uses `tl_mltl::past::evaluate_past`. The result is a `PastConformanceReport` with the two verdicts, or a typed reason when the contexts differ or the evaluator refuses a formula. One history and one anchor are evidence for that context only.

With the `infinite-trace` feature, `check_infinite_rewrite` evaluates the original and rewritten graphs on one lasso trace at a selected position through tl-mltl, and returns an `InfiniteConformanceReport`. A runtime comparison on one trace is evidence for that trace.

## Corpora

| Corpus | Contents |
| --- | --- |
| `corpus/rules/manifest.json` | One case per future-catalog rule: an input document, the expected rule and the expected outcome. The 38 enabled rules expect `pass`, and the 2 excluded rules expect `unsupported`. |
| `corpus/west-v1/` | A byte copy of the MIT-licensed WEST formula file and 10 selected published formulas. Each is represented as a tl-syntax graph in the integration tests, rewritten, and compared with tl-mltl. |
| `corpus/counterexamples/manifest.json` | 11 cases: 5 unsound rewrites that must produce a counterexample, 5 comparisons that must decline for a declared reason, and 1 malformed document. |

The three programs in `examples/` read these corpora and write one JSON row per case.

| Program | Checks |
| --- | --- |
| `rule_conformance` | Each catalog rule fires on its case, with the declared revision, and the rewrite agrees with the original on its whole bounded domain. Excluded rules report `unsupported`. |
| `counterexample_evidence` | Each mismatch case yields a witness trace that is then rebuilt and evaluated against both documents outside the enumeration that found it. A witness that does not separate the pair is a failure. Each non-conclusive case declines for its declared reason. |
| `normalization_sweep` | For a generated formula family: two identical requests give identical bytes, rewriting the output again reports `Unchanged`, the trace replays, the output validates under tl-syntax with the input's profile, and a zero work budget gives `BudgetExhausted` with no output. |

Row outcomes are `pass`, `fail`, `unsupported`, `not-computed`, `malformed` and `unavailable`. An expected refusal counts as `pass` for its case, and the row keeps the exact domain outcome.
