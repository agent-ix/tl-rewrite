---
id: SR-018
title: "Dependency review of the complete tl-rewrite specification corpus"
type: SpecReview
analysis: dependency
scope: "spec/**/*.md"
review_set: all
---

# SR-018: Dependency review of the complete tl-rewrite specification corpus

## Summary

The requirements state local dependencies and FR-007 declares its tl-syntax
contract dependency. The corpus does not contain the dependency-analysis DAG,
classification, or topological order required by this lens, so ordering between
shared enablement and repository feature work is not a maintained artifact.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1801 | medium | No dependency-analysis artifact classifies every FR/StR/NFR as enablement or feature, records prerequisite edges, and proves an acyclic implementation order; FR-006 and FR-007 consume shared releases but no corpus-owned ordering view records that boundary. | FR-001 through FR-007, NFR-001 through NFR-003 |
