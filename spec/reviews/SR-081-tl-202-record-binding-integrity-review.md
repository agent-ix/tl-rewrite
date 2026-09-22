---
id: SR-081
title: "integrity review of NFR-004-AC-9 (TL-202 per-gate record binding)"
type: SpecReview
analysis: integrity
scope: "spec/requirements/NFR-004-gate-set-integrity.md"
review_set: subset
---

## Summary

Checked AC-9's completeness (trace to StR/verification), consistency against
AC-3/AC-5 (no contradiction, correct layering), atomicity/testability
(`quire coverage --strict` shows NFR-004 at 8/9 backed, the one gap being
the pre-existing AC-8/Inspection pattern unrelated to this change), and the
integrity-analysis hidden-assumption probe table against AC-9. No probe
pattern applies (no new external CLI, multi-source lookup, pagination,
concurrency, scaffolding mode, or auth scope is introduced), and no defect
found.

## Findings

| ID      | Severity | Summary                          | Refs   |
| ------- | -------- | --------------------------------- | ------ |
| FND-001 | low      | AC-9 traces to NFR-004, which already carries `depends_on FR-006`, `extends NFR-003`, and `traces_to StR-001` in frontmatter; AC-9 needs no new relationship of its own since it is a mechanism within the same requirement, not a new requirement. `quire coverage` confirms `NFR-004-AC-9` is backed by real evidence symbols (`Trace: TC-064, NFR-004-AC-9` in `src/ci_guard.rs` and `tests/ci_guard.rs`), and TC-064 is a declared Test Matrix row. Complete. No change. | NFR-004-AC-9 |
| FND-002 | low      | AC-3 ("writes a ... record ... only on that recipe's own successful completion") and AC-9 (adds a second, independent condition — presenting the right token) compose as an AND, not a contradiction: both must hold for a record to be written, and AC-3 remains a true, if now partial, description of the write-gating behavior. The Scope section's item 4 explicitly frames AC-9 as additive ("binds each completion record to the specific gate recipe... " layered on item 3's reconciliation), so a reader is not left to infer the relationship. No change. | NFR-004-AC-3, NFR-004-AC-9 |
| FND-003 | low      | Hidden-assumption probe table: none of the six patterns (external CLI version pin, multi-source tie-breaking, pagination, concurrent/looped external calls, scaffolding interactive-vs-scripted mode, authenticated-API scopes) apply to AC-9 — it introduces no new external tool, network call, or auth boundary, only a data-flow change (target-specific Make variables) within the same `make`/`ci_guard` pair NFR-004 already governs. No gap. | NFR-004-AC-9 |
| FND-004 | low      | AC-9's mechanism newly leans on GNU Make's target-specific-variable `export` syntax, which is a GNU extension (not POSIX). This is not a *new* toolchain assumption this change introduces: the same Makefile already used `$(shell git rev-parse HEAD)` (also GNU-only) before this change, so the repository's implicit "this is GNU Make" assumption is pre-existing and merely deepened, not newly incurred. Confirmed empirically against this repository's actual `make` (positive and negative real-recipe runs both behaved as specified). No new NFR needed to pin a Make version — matches this document's existing silence on that same pre-existing dependency. No change. | NFR-004-AC-9 |

