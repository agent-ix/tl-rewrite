---
id: SR-070
title: "ears-conformance review of NFR-004 (TL-64 gate-set-integrity remediation)"
type: SpecReview
analysis: ears-conformance
scope: "spec/requirements/NFR-004-gate-set-integrity.md"
review_set: all
---

## Summary

Checked the Statement section against EARS grammar (ubiquitous / event /
state / unwanted-behavior / optional / complex) using `quire validate`'s
`ears:*` advisories as the objective check, not just reading by eye. The
first authored draft failed three advisories; the statement was restructured
into two single-`shall` sentences and now validates grammar-clean (139/139,
0 findings) alongside the rest of the repository's specs.

## Findings

| ID      | Severity | Summary                          | Refs   |
| ------- | -------- | --------------------------------- | ------ |
| FND-001 | medium   | Original Statement packed two `shall`s per sentence behind an em-dash aside ("the entry point SHALL control ... and SHALL fail closed"), which `quire validate` flagged `ears:non-singular`, `ears:unclassifiable`, and `ears:missing-subject` — the parenthetical broke subject-before-`shall` adjacency and the compound response broke pattern classification. Fixed by splitting into an unwanted-behavior sentence ("If ... then the CI entry point shall report `ci` as failed...") and a state-driven sentence ("While invoking Make, the CI entry point shall refuse to proceed if ..."), each with exactly one `shall` and the subject immediately preceding it. Re-validated clean. | NFR-004 |
| FND-002 | low      | The original inverted phrasing ("SHALL refuse ... unless X") is grammatically valid but reads as a double negative next to the unwanted-behavior form used elsewhere in this file (AC rows use "If ... then"). Rewriting the first sentence as an explicit trigger/response pair removed the inversion and made the two lead sentences structurally parallel, which also made the non-singular defect easier to spot. | NFR-004 |
