# API reference

The crate root re-exports the items below. The module paths are `catalog`, `engine::{future, past}`, `equivalence`, `infinite`, `replay`, `report` and `ci_guard`. Records derive `serde` and use `camelCase` field names on the wire. Enums such as `RewriteStatus` serialize in `snake_case`.

## Functions

| Item | Purpose |
| --- | --- |
| `rewrite(input, formula_id, options, source_revision)` | Rewrite a validated `FormulaDocument` and return a `RewriteReport`. |
| `rewrite_with_context(input, formula_id, options, source_revision, signal_catalog, requirement_context)` | The same, bound to a `SignalCatalogDocument` and an optional `RequirementContextDocument`. Returns a v2 report. |
| `replay(input, expected)` | Re-run a report's request and return a `ReplayReport`. |
| `replay_with_context(input, expected, signal_catalog, requirement_context)` | Replay a contextual report. |
| `check_equivalence(original, rewritten, comparison_id, options)` | Exhaustive bounded comparison of two closed-trace formulas. |
| `check_equivalence_with_context(...)` | The same, with a signal catalog and requirement context. |
| `check_past_equivalence(original, rewritten, comparison_id, limits)` | Compare two `PastEvaluationContext` values with the tl-mltl past evaluator. |
| `rewrite_infinite(input, fairness, options, source_revision, max_report_bytes)` | Rewrite an `InfiniteFormulaDocument` and remap fairness roots. |
| `replay_infinite(input, fairness, expected, trusted_source_revision)` | Re-execute an `InfiniteRewriteReport`; returns a Boolean. |
| `check_infinite_rewrite(...)` | Compare original and rewritten infinite graphs on one lasso trace (feature `infinite-trace`). |
| `catalog()`, `past_catalog()`, `infinite_catalog()` | The immutable rule catalogs. |
| `report::read(bytes, input, limits)`, `report::read_with_context(...)` | Strict-read a canonical report, then replay it. |

## Rewrite types

| Item | Purpose |
| --- | --- |
| `RewriteOptions` | `strategy` and `budgets`. `Default` gives `BottomUpFirstMatch` and the default budgets. |
| `RewriteStrategy` | `BottomUpFirstMatch`. |
| `RewriteBudgets` | `max_iterations` 32, `max_nodes` 100,000, `max_rule_applications` 100,000, `max_work_units` 1,000,000 by default. |
| `BudgetKind` | `Iterations`, `Nodes`, `RuleApplications`, `WorkUnits`. |
| `RewriteStatus` | `Unchanged`, `Normalized`, `BudgetExhausted`, `NonConvergent`, `InvalidInput`, `UnsupportedProfile`, `UnresolvedBinding`. |
| `RewriteReport` | Identities and digests, options, counters, `steps`, `status`, `exhausted_budget`, `detail`, `binding_failure` and `output`. Schemas `tl-rewrite.report/v1` and `v2`. |
| `RewriteStep` | `sequence`, `pass`, `source_node`, `source_span`, `rule_id`, `rule_revision`, `before_sha256`, `after_sha256`, `intermediate_sha256`. |
| `BindingFailure`, `BindingLocus` | The first unbound proposition of a contextual request, and whether it was in the `Input` or the `Output`. |
| `InfiniteRewriteReport` | Infinite-trace report with identities, `steps`, `output`, `output_fairness`, and `failure`. `succeeded()`. |
| `InfiniteRewriteFailure` | `IdentityMismatch`, `UnmappableFairness`, `Budget(BudgetKind)`, `ReportBytes`, `NonConvergent`. |
| `MAX_INFINITE_REPORT_BYTES` | 64 MiB ceiling for one infinite report. |

## Catalog types

| Item | Purpose |
| --- | --- |
| `CatalogDocument` | `schema_version`, `catalog_version`, `catalog_sha256` and ordered `rules`. |
| `RuleDefinition` | `id`, `revision`, `class`, `disposition`, `semantic_profiles`, `precondition`, `provenance`, `exclusion_reason`. |
| `RuleClass` | `Normalization`, `Simplification`, `Negation`, `Temporal`. |
| `RuleDisposition` | `Enabled` or `Excluded`. |
| `Provenance`, `ProvenanceKind` | The cited authority: `StatedDerivation` or `PrimarySource`, with a URI, a locator and a statement. |
| `WEST_REVISION` | The WEST source revision that the selected fixtures come from. |

## Replay and record types

| Item | Purpose |
| --- | --- |
| `ReplayReport` | `status`, the SHA-256 of the expected and observed reports, and for contextual replay the catalog digest and context. Schemas `tl-rewrite.replay/v1` and `v2`. |
| `ReplayStatus` | `Verified` or `Mismatch`. |
| `RecordLimits` | `document_bytes`, `json_depth`, `string_bytes`. `OWNER_MAXIMA`, `constrained()`; `Default` is the owner maxima. |
| `RecordReadError` | `code()`, `resource()` and a detail. |
| `RecordReadErrorCode` | `ResourceLimit`, `Malformed`, `NonCanonical`, `ExpectedMismatch`. Non-exhaustive. |

## Equivalence types

| Item | Purpose |
| --- | --- |
| `ConformanceOptions` | `max_propositions` 4, `max_horizon` 8, `max_traces` 1,000,000 by default. |
| `ConformanceReport` | `status`, `reason`, `horizon`, `trace_length`, `total_traces`, `traces_checked`, `counterexample`, both verdicts and a `limitation` string. Schemas `tl-rewrite.conformance/v1` and `v2`. |
| `ConformanceStatus` | `Equivalent`, `Mismatch`, `NonConclusive`. |
| `ConformanceReason` | `UnsupportedProfile`, `PropositionLimit`, `HorizonLimit`, `TraceDomainLimit`, `EvaluatorError`, `OriginalBinding`, `RewrittenBinding`. |
| `PastEvaluationContext` | A formula, its identity, a position history, an anchor and a proposition-map identity. |
| `PastConformanceReport`, `PastConformanceReason` | Result of a past comparison and its typed non-conclusive reason. Schema `tl-rewrite.past-conformance/v2`. |
| `InfiniteConformanceReport`, `InfiniteConformanceStatus`, `InfiniteConformanceReason` | Result of an infinite comparison (feature `infinite-trace`). |

## Disposition mapping

`rewrite_disposition`, `conformance_disposition` and, with `infinite-trace`, `infinite_conformance_disposition` map a rewrite or comparison outcome to an external result class. The class type is `Fr341Disposition`. Its variants are `NoTemporalVerdict`, `Unsupported`, `ResourceIncomplete` and `Failed`. A successful rewrite or a completed comparison maps to `NoTemporalVerdict`, so neither is reported as a temporal verdict. `DispositionMappingError` reports a status and reason pair that do not agree.

## Build gate

The `ci_guard` module and the `ci_guard` binary run the local gate. `make guarded-ci` builds the binary, refuses to run Make when the Makefile or the environment could suppress a failing prerequisite, and then compares the gates that wrote a completion record with the declared list.
