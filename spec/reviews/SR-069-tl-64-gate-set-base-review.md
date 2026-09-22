---
id: SR-069
title: "base review of NFR-004 (TL-64 gate-set-integrity remediation)"
type: SpecReview
analysis: base
scope: "spec/requirements/NFR-004-gate-set-integrity.md, spec/requirements/NFR-003-qualification-integrity.md, spec/spec.md"
review_set: all
---

## Summary

Base checklist pass over the new NFR-004 and its two ripple edits (NFR-003's
Scope handoff note, spec.md's Requirements Architecture line). ID format,
frontmatter schema, and cross-references validate clean under `quire
validate --strict`. No blocking findings; one informational note on test
coverage timing.

## Findings

| ID      | Severity | Summary                          | Refs   |
| ------- | -------- | --------------------------------- | ------ |
| FND-001 | low      | NFR-004's Acceptance Criteria carry no TC-XXX references yet, unlike NFR-003's AC table (which cites existing tests). This is expected pre-implementation state, not a defect: no code or tests exist for this control yet, and this task stops before implementation. `spec-to-plan` and, later, the test matrix are the right place to allocate TC ids. | NFR-004 |
| FND-002 | low      | Relationship verbs on NFR-004 (`depends_on` to FR-006, `extends` to NFR-003, `traces_to` to StR-001) are drawn from the NFR archetype's `allowed_links` vocabulary in `manifest.yaml`, not copied uncritically from sibling NFRs; `refines`/`implements` were tried first and rejected by `quire validate` as disallowed edge types for NFR sources, then corrected. Recorded here so the correction is traceable to a real validator finding rather than an unexplained diff. | NFR-004 |
