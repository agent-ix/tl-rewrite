---
id: SR-002
title: Code review of tl-rewrite v0.1 implementation
type: SpecReview
analysis: code-review
scope: src/**/*.rs, tests/**/*.rs, corpus/**/*, schemas/**/*.json, scripts/**/*, Cargo.toml, Makefile, .github/workflows/ci.yml
review_set: all
---

# Code review of tl-rewrite v0.1 implementation

## Summary

The agent code review examined catalog/implementation correspondence, semantic
profiles, topological remapping, source-span retention, first-match order,
iteration/application/node/work budgets, partial-state handling, cycle checks,
trace chaining, replay substitution, horizon/domain arithmetic, WEST
provenance, unsafe usage, and CI. Three gaps were corrected during review:
enabled rules were narrowed to closed-trace v1 until a prefix population exists;
the replay report gained a request digest binding options and catalog identity;
and equivalence materialization gained a process-safe instant ceiling even when
a caller raises its declared horizon budget. No unresolved code defect or
uncovered blocking requirement was found. Human review remains mandatory.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-201 | medium | Resolved: rule applicability now rejects online-prefix input because only the closed-trace population has retained v1 equivalence evidence. | FR-001, FR-002, FR-004 |
| FND-202 | medium | Resolved: replay now binds input, identity, source, catalog, strategy, and budgets in `requestSha256`, and mutation tests reject changed options and intermediates. | FR-003-AC-2 |
| FND-203 | medium | Resolved: trace materialization fails non-conclusively above a fixed process-safe instant ceiling, independent of caller-selected domain ceilings. | FR-004-AC-2, NFR-001-AC-2 |
| FND-204 | low | All 38 enabled rules have individual exhaustive small-domain tl-mltl agreement; the two primary-source WEST Theorem 3 rules remain visible but non-executable. | FR-001, FR-004 |
