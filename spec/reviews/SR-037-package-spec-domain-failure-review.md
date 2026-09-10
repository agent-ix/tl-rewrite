---
id: SR-037
title: "Failure-domain review of the hosted ix-flow package-specification domain"
type: SpecReview
analysis: failure-domain
scope: "NFR-003-AC-7 through NFR-003-AC-12 and TC-039 for tl-rewrite issue 33"
review_set: all
---

## Summary

The failure-domain review covered identity confusion between registry, alias,
git, URL, tarball/file, workspace/link, and dynamic package arguments plus YAML
metadata and trigger topology. The only uncovered bypass was a non-literal
argument whose value could be supplied through an environment or shell
expansion; AC-12 now makes that state an error.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-3701 | medium | **FIXED:** the literal ix-flow census could remain satisfied while a second install argument was computed by shell substitution or workflow interpolation. Every consumed package argument must now be one statically classified literal. | NFR-003-AC-7, NFR-003-AC-12, TC-039 |
| FND-3702 | low | No topology, extension-point, evaluation-purity, or persistent-entity identity surface is introduced; the control is a bounded read-only inspection of one tracked workflow. | NFR-003, TC-039 |

## Failure Boundaries

An empty run-script population, a missing expected specification, an alternate
or duplicate specification, a non-literal argument, and an automatic trigger
are all non-success states. Comments and metadata remain outside execution
unless a run script attempts to consume them dynamically, which AC-12 refuses.
