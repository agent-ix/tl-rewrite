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

Applies to `MAX_INFINITE_REPORT_BYTES`, the private
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
| NFR-006-AC-2 | For `MAX_INFINITE_REPORT_BYTES` and the `max_report_bytes` argument, a value equal to the limit is accepted; a value one past the limit is refused with the typed error that names that limit. The refusal is `InfiniteRewriteFailure::ReportBytes`. A caller value above the constant is clamped to the constant. The native run uses the owner value and is never skipped. The Miri run uses a caller-lowered `max_report_bytes` for the same code path. | Test (TC-078) |
| NFR-006-AC-3 | For `RecordLimits::OWNER_MAXIMA` document bytes, JSON depth, and string bytes, a value equal to the limit is accepted; a value one past the limit is refused with the typed error that names that limit. The refusal is `RecordReadErrorCode::ResourceLimit`. A caller value above a ceiling is clamped to it. The native run uses the owner value and is never skipped. The Miri run uses a caller-lowered limit for the same code path, for this limit: document bytes. Depth and string ceilings are small and run at the owner value under Miri. | Test (TC-079) |
| NFR-006-AC-4 | For `ConformanceOptions` propositions, horizon, and traces, and for `MAX_MATERIALIZED_INSTANTS`, a value equal to the limit is accepted; a value one past the limit is refused with the typed error that names that limit. The refusals are `PropositionLimit`, `HorizonLimit`, and `TraceDomainLimit`. `MAX_MATERIALIZED_INSTANTS` has no public lowering input, so under Miri it is a named constant defined by `cfg(miri)` with a smaller value, and the test reads the constant by name. The native run uses 100_000 and is never skipped. | Test (TC-080) |
| NFR-006-AC-5 | Every variant of the enums named in Scope is either returned by at least one test that exercises the producing API, not only a mapping table, or removed from the enum in the same change. | Test (TC-081) |
| NFR-006-AC-6 | The unit and property suites complete under Miri on the pinned nightly named in the Makefile, with zero undefined-behavior reports, using a documented proptest case count and isolation setting. | Test (TC-082) |

## Dependencies

NFR-001 and NFR-005 supply the limits being proved. tl-syntax owns the graph
ceilings.
