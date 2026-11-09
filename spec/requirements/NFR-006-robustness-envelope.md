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
`DispositionMappingError`, and `ViolationKind`. tl-rewrite is a `std` crate and
makes no `no_std` claim, although it consumes tl-syntax in `alloc` mode; the
embedded build obligation belongs to tl-syntax.

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
| NFR-006-AC-1 | Each `RewriteBudgets` field (iterations, nodes, rule applications, work units) is accepted at its value and refused at one below the needed amount with `RewriteStatus::BudgetExhausted` and the matching `BudgetKind`, on the finite, past, and infinite paths. | Test |
| NFR-006-AC-2 | `MAX_INFINITE_REPORT_BYTES` and the `max_report_bytes` argument admit a report of exactly that size and refuse one byte more with `InfiniteRewriteFailure::ReportBytes`, and a caller value above the constant is clamped to it. | Test |
| NFR-006-AC-3 | `RecordLimits::OWNER_MAXIMA` document bytes, JSON depth, and string bytes admit a record at the ceiling and refuse one above with `RecordReadErrorCode::ResourceLimit`, and no caller value raises a ceiling. | Test |
| NFR-006-AC-4 | `ConformanceOptions` propositions, horizon, and traces, and `MAX_MATERIALIZED_INSTANTS`, admit a domain at the limit and refuse one above with `PropositionLimit`, `HorizonLimit`, or `TraceDomainLimit`. | Test |
| NFR-006-AC-5 | Every variant of the enums named in Scope is returned by at least one test that exercises the producing API, not only a mapping table. | Test |
| NFR-006-AC-6 | The unit and property suites complete under Miri with zero undefined-behavior reports, using a documented proptest case count and isolation setting. | Test |
| NFR-006-AC-7 | tl-rewrite documentation and manifest make no `no_std` claim and state that the crate requires `std`. | Test |

## Dependencies

NFR-001 and NFR-005 supply the limits being proved. tl-syntax owns the graph
ceilings and the `no_std` build.
