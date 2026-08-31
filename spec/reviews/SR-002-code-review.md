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
| FND-209 | medium | Resolved: evidence behavior rejects skipped or failed governance checks, parameter identity includes semantic implementation and verification files, every outer manifest has an executable committed anchor, manifest-to-artifact sizes/digests are rechecked, and each retained summary is rederived from status files. | FR-005, NFR-002 |
| FND-210 | low | Resolved: the engine has no unreachable catalog-lookup failure or unowned `invalid_catalog` wire disposition; the enabled-rule population exercises all 38 maintained identities and asserts every emitted step revision equals its catalog revision. | FR-001, FR-002 |
| FND-211 | medium | Resolved: retained trace provenance uses a single budgeted reachability walk over per-source retained operands instead of transitive `BTreeSet` unions. A 100,000-node rewriting Not-chain completed in 313 ms in the local release probe (previous review: 175.34 s), and TC-021 proves both unique visitation and work-budget charging. | FR-002, NFR-001 |
| FND-212 | high | Resolved: summary rederivation is unconditional, the one pre-summary legacy record is explicitly migrated as inconclusive, added status files enter the outcome census, and the outer checksum manifest is enforced as the exact record membership boundary. | FR-005, NFR-002 |
