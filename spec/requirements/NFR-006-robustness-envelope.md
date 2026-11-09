---
id: NFR-006
title: Prove declared limits, error variants, and memory safety under test
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/tl-rewrite/NFR-001
    type: constrains
  - target: ix://agent-ix/tl-rewrite/NFR-005
    type: constrains
---

# NFR-006: Prove declared limits, error variants, and memory safety under test

## Statement

tl-rewrite shall accept input exactly at every declared resource limit, refuse
it one unit above with that limit's own typed status, reason, or error, produce
every variant of every public status, reason, failure, and error enum from at
least one test through the public API, and run its unit and property suites
without undefined behavior under Miri.

## Scope

Applies to `MAX_INFINITE_REPORT_BYTES`, the crate-private
`MAX_MATERIALIZED_INSTANTS`, `RecordLimits::OWNER_MAXIMA`, `RewriteBudgets`,
`ConformanceOptions`, and the `max_report_bytes` argument. The public enums are
`RewriteStatus`, `BudgetKind`, `InfiniteRewriteFailure`, `BindingLocus`,
`RecordReadErrorCode`, `ConformanceStatus`, `ConformanceReason`,
`PastConformanceReason`, `InfiniteConformanceStatus`,
`InfiniteConformanceReason`, `ReplayStatus`, `Fr341Disposition`,
`DispositionMappingError`, and `ViolationKind`. tl-rewrite is a `std` crate.

## Rationale

A mapping-table test that lists a variant proves the table, not that the engine
can emit the variant. A hard constant or owner maximum that no test touches can
drift without a failure. Miri checks that the tested surface stays free of
undefined behavior in dependencies the crate cannot audit.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Declared limits lacking an at-limit accept test or a limit-plus-one typed refusal test | 0 | 0 | Test |
| Public status, reason, failure, and error variants not produced by any test | 0 | 0 | Test |
| Undefined-behavior reports from the unit and property suites under Miri | 0 | 0 | Test |

## Verification

One test per declared limit builds an input at the limit and at limit plus one.
One test per enum drives the public API to each variant rather than listing it
in a mapping table. The Miri target runs the unit and property suites with
reduced proptest case counts.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-006-AC-1 | For each `RewriteBudgets` field (iterations, nodes, rule applications, work units), on the finite, past, and infinite paths: a budget equal to the need succeeds; a budget one less returns `RewriteStatus::BudgetExhausted` with the matching `BudgetKind`. | Test (TC-077) |
| NFR-006-AC-2 | For `MAX_INFINITE_REPORT_BYTES` and the `max_report_bytes` argument, a value equal to the limit is accepted; a value one past the limit is refused with the typed error that names that limit. The refusal is `InfiniteRewriteFailure::ReportBytes`. A caller value above the constant is clamped to the constant. A report of `MAX_INFINITE_REPORT_BYTES` bytes needs a graph beyond any budget a test can spend, so the native and Miri runs both exercise the boundary at a caller-lowered `max_report_bytes` for the same code path and assert only the clamp at the owner value. | Test (TC-078) |
| NFR-006-AC-3 | For `RecordLimits::OWNER_MAXIMA` document bytes, JSON depth, and string bytes, a value equal to the limit is accepted; a value one past the limit is refused with the typed error that names that limit. The refusal is `RecordReadErrorCode::ResourceLimit`. A caller value above a ceiling is clamped to it. The native run uses the owner value and is never skipped. The Miri run uses a caller-lowered limit for the same code path, for this limit: document bytes. Depth and string ceilings are small and run at the owner value under Miri. The JSON parser's own recursion limit equals the owner depth, so one past the owner depth is refused as `Malformed`; the typed `ResourceLimit` depth refusal is proved at a caller-lowered ceiling on native and Miri. | Test (TC-079) |
| NFR-006-AC-4 | For `ConformanceOptions` propositions, horizon, and traces, and for `MAX_MATERIALIZED_INSTANTS`, a value equal to the limit is accepted; a value one past the limit is refused with the typed error that names that limit. The refusals are `PropositionLimit`, `HorizonLimit`, and `TraceDomainLimit`. `MAX_MATERIALIZED_INSTANTS` is crate-private with no lowering input, so its boundary is tested by an in-crate test in `src/equivalence.rs` that reads the constant by name; under Miri the constant is a smaller `cfg(miri)` value and the same test runs in `--lib`. The native run uses 100_000 and is never skipped. | Test (TC-080) |
| NFR-006-AC-5 | Every variant of the enums named in Scope is either returned by at least one test that exercises the producing API, not only a mapping table, or removed from the enum in the same change. `RewriteStatus::Failed`, `InfiniteRewriteFailure::Internal`, `ConformanceReason::InvalidInput`, and `InfiniteConformanceReason::InvalidTrace` were removed because no input can produce them through a validated document: the former fail-closed arms are `expect` on construction-time invariants or total indexing, and a rebuilt graph past the owner node or depth ceiling is `Budget(Nodes)`. A deserialized `FairnessPremisesDocument` is checked for root order and duplicates but not against the graph, so `rewrite_infinite` (and therefore `replay_infinite` and `check_infinite_rewrite`) refuses a root outside the graph as `InfiniteRewriteFailure::IdentityMismatch`, and a test deserializes such a document to assert it. A variant whose producing arm sits behind a crate-private seam is returned by a unit test of the owning module instead: `RewriteStatus::NonConvergent`, `InfiniteRewriteFailure::NonConvergent`, `BindingLocus::Output`, `InfiniteConformanceReason::ProviderFailure`, and every `ViolationKind`. | Test (TC-081) |
| NFR-006-AC-6 | The unit, property, corpus, oracle and replay suites complete under Miri on the pinned nightly named in the Makefile, with zero undefined-behavior reports, using a documented proptest case count and `-Zmiri-disable-isolation` (the corpus suites read repository files). The Miri suites are `--lib`, `robustness`, `property`, `infinite_disposition`, `catalog`, `wire_records`, `rewrite`, `rule_evidence`, `replay`, `past_history_corpus`, `infinite_conformance`, `oracle_finite_rewrite`, `infinite_owner_corpus`, `oracle_equivalence_properties` and `tc_053_profile_subsystems`, with `ci_guard::` tests skipped because they drive Make. Every other suite is excluded for one stated reason. Process or file system: `ci_guard`, `doc_claims`, `infinite_fuzz_seeds`, `infinite_rules` and `equivalence` start processes or walk the repository, which Miri cannot interpret. Measured too slow, with no lowerable limit: `past_rewrite` exceeds 70 minutes for the single test `every_admitted_past_fold_preserves_event_and_fixed_sample_verdicts`, and `future_lowering_parity` and `contextual_rewrite` each exceed 15 minutes (the trial timeout) over a fixed corpus. Measured Miri times for the included corpus suites are replay 119 s, past_history_corpus 99 s, infinite_conformance 320 s, oracle_finite_rewrite 280 s, infinite_owner_corpus 557 s, oracle_equivalence_properties 85 s at 4 cases and tc_053_profile_subsystems 358 s. All excluded suites run natively. `the_evaluator_error_reason_is_returned_for_an_over_deep_graph` is ignored under Miri, so `ConformanceReason::EvaluatorError` and every `ViolationKind` are covered natively. Where a limit needs an input too large to interpret, the Miri run substitutes a caller-lowered limit on the same code path, as the ACs above state. | Test (TC-082) |

## Dependencies

NFR-001 and NFR-005 supply the limits being proved. tl-syntax owns the graph
ceilings.
