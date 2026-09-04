---
id: SR-012
title: Composite review of post-merge residual requirements
type: SpecReview
analysis: base
scope: issue #19, FR-006-AC-7, TC-027, TC-029, NFR-003-AC-3
review_set: all
---

# Composite review of post-merge residual requirements

## Summary

The residuals are narrow strengthening work, not a new assurance subsystem.
They require the live removal census to expose its enumeration and exemptions to
retained negative controls, and require the scratch probe's containment check to
compare resolved locations when those locations exist.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-1201 | medium | The production Git enumeration needs a real repository fixture so preferred makefiles, workflows, excludes, and enumeration failure exercise the same helper as the live census. | FR-006-AC-7, TC-029 |
| FND-1202 | medium | The deleted-reference implementation and its adjacent expected copy are not independent. A labelled, unsealed authorial cross-check makes the two lists separately reviewable without overstating authority. | FR-006-AC-7, TC-029 |
| FND-1203 | low | The complete `HistoricalProse` exemption population must equal the intended `.md` population under exactly two trees. | FR-006-AC-7, TC-029 |
| FND-1204 | low | The probe's real-store leaf must be canonicalized when present; only absence permits lexical fallback. The post-creation `!is_symlink` assertion is dead after the ownership-first check. | TC-027, NFR-003-AC-3 |
| FND-1205 | medium | Full Make execution-control qualification is explicitly outside this repository-scoped plan and remains open in #11 / Engineering Assurance #11. | NFR-003 |
