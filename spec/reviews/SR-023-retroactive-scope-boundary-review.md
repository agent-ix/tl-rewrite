---
id: SR-023
title: "Scope and boundary review of the complete tl-rewrite specification corpus"
type: SpecReview
analysis: scope-boundary
scope: "spec/**/*.md"
review_set: all
---

# SR-023: Scope and boundary review of the complete tl-rewrite specification corpus

## Summary

The master specification and architecture description define clear exclusions
for parsing, contract-IR translation, generic evidence retention, production
monitoring, and release authority. The corpus does not provide the required
per-requirement responsibility allocation or an assumed-versus-guaranteed
external-dependency table.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2101 | medium | No scope-boundary artifact assigns every FR/StR/NFR one owning component and responsibility class, or classifies tl-syntax, tl-mltl, Quire, Quoin, Engineering Assurance, and WEST as assumed or guaranteed with named contracts. | MRS-001, AD-001, FR-004, FR-006, FR-007 |
