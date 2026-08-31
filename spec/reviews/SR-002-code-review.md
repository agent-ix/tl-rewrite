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
iteration/application/node/work budgets, compaction, partial-state handling, cycle checks,
trace chaining, replay substitution, horizon/domain arithmetic, WEST
provenance, unsafe usage, and CI. Three gaps were corrected during review:
enabled rules were narrowed to closed-trace v1 until a prefix population exists;
the replay report gained a request digest binding options and catalog identity;
and equivalence materialization gained a process-safe instant ceiling even when
a caller raises its declared horizon budget. Pull-request review then prompted
exact upstream repins, nonzero-lower-bound temporal properties, indexed reachable
graph compaction, output-relevant traces, catalog-derived step revisions, and direct
engine-level cycle/error regressions. Retained summaries are rederived from their
status files, and every authored Rust evidence symbol is trace-tagged. No unresolved code defect or uncovered blocking
requirement was found. Human review remains mandatory.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-201 | medium | Resolved: rule applicability now rejects online-prefix input because only the closed-trace population has retained v1 equivalence evidence. | FR-001, FR-002, FR-004 |
| FND-202 | medium | Resolved: replay now binds input, identity, source, catalog, strategy, and budgets in `requestSha256`, and mutation tests reject changed options and intermediates. | FR-003-AC-2 |
| FND-203 | medium | Resolved: trace materialization fails non-conclusively above a fixed process-safe instant ceiling, independent of caller-selected domain ceilings. | FR-004-AC-2, NFR-001-AC-2 |
| FND-204 | low | All 38 enabled rules have individual exhaustive small-domain tl-mltl agreement; the two primary-source WEST Theorem 3 rules remain visible but non-executable. | FR-001, FR-004 |
| FND-205 | high | Resolved: exact amended tl-syntax/tl-mltl revisions are pinned, Until/Release fixtures include `a > 0`, and both git sources are denied except for time-boxed exact-project exceptions that block release. | FR-001, FR-004, AA-001 |
| FND-206 | medium | Resolved: each pass interns identical kind/span nodes through deterministic logarithmic-time indexes, prunes unreachable nodes, enforces the node ceiling on the compacted graph, and directly tests repeated-state non-convergence through the engine plus nondegenerate limits. | FR-002 |
| FND-207 | medium | Resolved: discarded subtree candidates are absent from traces and application budgets; every retained step takes its revision from the matching catalog definition. | FR-003 |
| FND-208 | medium | Resolved: WEST source lines are bound to exact typed formula-document digests and evaluator failures have a direct non-conclusive regression. | FR-004 |
| FND-209 | medium | Resolved: evidence behavior rejects skipped or failed governance checks, parameter identity includes semantic implementation files, checksummed retained evidence is part of the complete gate, and verification rederives each retained summary from status files. | FR-005, NFR-002 |
| FND-210 | low | Resolved: an executable identity absent from the catalog returns typed `invalid_catalog` without panicking; the enabled-rule population proves all 38 maintained identities resolve and execute. | FR-001, FR-002 |
