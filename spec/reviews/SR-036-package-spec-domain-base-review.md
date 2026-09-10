---
id: SR-036
title: "Base review of the hosted ix-flow package-specification domain"
type: SpecReview
analysis: base
scope: "NFR-003-AC-7 through NFR-003-AC-12 and TC-039 for tl-rewrite issue 33"
review_set: all
---

## Summary

The base checklist reviewed identifier continuity, atomicity, normal and adverse
paths, and the six coverage rules for the TC-039 correction. The initial draft
was repaired by splitting five bundled outcomes and making dynamic or otherwise
unclassifiable install arguments fail closed; the resulting six criteria share
one P0 integration test without leaving an unowned branch.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-3601 | medium | **FIXED:** the initial NFR-003-AC-7 draft bundled package population, alternate-spec rejection, metadata exclusion, trigger policy, and runtime version; these are now atomic AC-7 through AC-11. | NFR-003-AC-7, NFR-003-AC-8, NFR-003-AC-9, NFR-003-AC-10, NFR-003-AC-11, TC-039 |
| FND-3602 | medium | **FIXED:** a second package supplied through shell or workflow expansion was outside the initial literal-token population; AC-12 now requires every consumed install argument to be statically classifiable or refused. | NFR-003-AC-12, TC-039, tl-rewrite#33 |

## Checklist Result

- NFR, acceptance-criterion, and test-case identities are unique; AC-7 through
  AC-12 continue the active allocation without reusing removed AC-4.
- TC-039 names normal, alternate-family, duplicate, non-literal, inert-metadata,
  trigger, and runtime outcomes and remains planned until implementation.
- No user-authored language, grammar, temporal source profile, or release
  authority is introduced.
