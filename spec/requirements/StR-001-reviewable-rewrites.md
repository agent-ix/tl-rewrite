---
id: StR-001
title: Provide reviewable semantics-preserving rewrites
type: StR
---

# StR-001: Provide reviewable semantics-preserving rewrites

## Stakeholder Need

Temporal-tool developers need formula simplification whose semantics,
preconditions, source, and exact transformations can be independently reviewed
without trusting undocumented implementation behavior.

## Rationale

A smaller formula is harmful if a profile-dependent identity or stale rule
revision silently changes its finite-trace meaning.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-001-VC-1 | Every enabled rule identifies a derivation or primary source, profiles, preconditions, and revision. | Inspection (TC-019) |
| StR-001-VC-2 | A reviewer can replay every successful output-changing step and detect substituted inputs, rules, options, or intermediates. | Test (TC-010) |

## Stakeholders

Temporal-crate developers, assurance reviewers, and the human source-release owner.

## Context and Assumptions

Inputs have passed the exact tl-syntax structural contract.

## Traceability

This need is realized by FR-001, FR-003, and FR-005 and verified by TM-001.
