---
id: SR-006
title: Gap analysis of the tl-rewrite shared assurance migration
type: SpecReview
analysis: gap-analysis
scope: spec/**/*.md, assurance/**/*, scripts/**/*, examples/**/*.rs, tests/**/*.rs, corpus/**/*, evidence/**/*, Makefile, .github/workflows/ci.yml
review_set: all
---

# Gap analysis of the tl-rewrite shared assurance migration

## Summary

Every row of the test matrix is backed. `quire coverage --scope . --strict`
reports 72 of 72 rows backed with no status lies, over 39 evidence symbols,
after the migration added TC-022 through TC-030 and retired TC-018, whose only
subject was the local evidence framework.

The dual run is recorded as observed. It is not a parity result and it is not
green at the base.

## Dual run, recorded as observed

| Path | Revision | Result |
|---|---|---|
| old — `make ci` from the base Makefile plus the local evidence scripts | `74aa3f1` (base) | **exit 2**. Eleven of twelve prerequisites pass; `verify-evidence` fails with `assured evidence is not v2-qualified`, because the active retained record's own source revision does not carry the machinery `evidence_profile.resolve_profile` requires. 29 Rust tests pass. |
| old — same Makefile and scripts, run at the migration revision | `473bd97` | **exit 2** at `check-failure-propagation`: the migrated Makefile has none of the targets the guard requires. Individually: `verify_evidence.sh` exit 1 (`assurance argument does not identify one record, digests, and outcome count`), `run_policy_tests.py` exit 1, `tool_identity.py --verify-live` exit 1 (host lock digests), `check_traceability_coverage.py` exit 0 (72/72). |
| new — `make ci` with both paths present | `473bd97` | **exit 2**, on one assertion only: TC-029, which requires the local framework to be gone. Every other gate green; 36 of 37 Rust tests pass. |
| new — `make ci` after the deletion commit | `194c8ae` (head) | **exit 0**. 39 Rust tests pass. |

No green baseline was manufactured. The base was already red for a reason this
migration deletes rather than fixes, and that is stated rather than smoothed
over.

## Legacy compatibility, measured

All 11 retained envelopes declare `quire.derivation-evidence/v1`. The pinned
mapping covers `quire.pgm01-evidence` v1 and v2 and returns `incompatible` with
the reason *"unknown PGM-01 schema version"* for every one of them. Measured by
running `map_pgm01_bytes` over this repository's own bytes, not inherited: the
answer differs per repository across this campaign, and `quire-analyze`'s
Markdown artifacts read `unreadable` rather than `incompatible`.

The census reads 582 files under `evidence/`, moves none of them, and asks Git
whether the retained bytes are the committed bytes. Two positive controls from
the release's own fixtures are accepted, so the mapping is observed to accept as
well as to refuse.

## Coverage and its known blind spot

72 of 72 rows backed, 0 status lies, 39 of 39 evidence symbols tagged and bound.

The blind spot is upstream and out of this repository's control:
`quire coverage --strict` reports `status-column-matches-nothing` and exits 0,
because the configured status column and the archetype's asserted columns are
mutually exclusive in a validated repository. A fabricated row *is* caught; a
mis-*statused* row is not. Tracked program-wide as
`agent-ix/quire-contract-ir#21`. Cite that when relying on the 72/72 figure; it
is a backing figure, not a status-verified one.

## Requirement-level gaps

| Requirement | Backed by | Gap |
|---|---|---|
| FR-006-AC-1 | TC-023 | none |
| FR-006-AC-2 | TC-024, TC-030 | none |
| FR-006-AC-3 | TC-025 | none |
| FR-006-AC-4 | TC-026 | none |
| FR-006-AC-5 | TC-027 | none |
| FR-006-AC-6 | TC-028 | none |
| FR-006-AC-7 | TC-029 | none |
| NFR-003-AC-1..5 | TC-024, TC-026, TC-027, TC-030 | none |

## Findings

Open gaps, carried rather than closed. Every one is DEFERRED to a linked issue or
ACCEPTED with an explicit rationale; none is silently absorbed.

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-601 | medium | DEFERRED to agent-ix/tl-rewrite#11. The Make execution-control guard is gone. `.IGNORE:` takes `make ci` from exit 2 to exit 0 with all 13 prerequisites reporting success; the structural backstop covers only the work re-run inside `assurance-inputs`. Recorded in NFR-003, AA-001, the change-assurance declaration and the Makefile header. | NFR-003 |
| FND-602 | medium | DEFERRED to agent-ix/tl-rewrite#10. The `tl-syntax` pin cannot move to the merged `953ee825` while `tl-mltl` pins `740182f1`; both crates must resolve the same revision. Measured as E0308 at four call sites in `src/equivalence.rs`. | AA-001, NFR-002 |
| FND-603 | medium | DEFERRED to agent-ix/engineering-assurance#21. The pinned mapping has no reader for `quire.derivation-evidence/v1`, so all 11 retained envelopes are refused. Reported as the compatibility result, not worked around. | FR-006-AC-4 |
| FND-604 | low | DEFERRED to agent-ix/engineering-assurance#20. engineering-assurance v0.2.0 records `pending_human_acceptance` and ships no `human_acceptance_recorded` predicate. Reported, never gated on in either direction. | FR-006-AC-1 |
| FND-605 | low | ACCEPTED. No `ix-flow` decision event exists for this change, so the receipt reads `incomplete` with a missing decision. That is the correct state for an unreviewed change, only the repository owner may create one, and the chain demonstrates it as the `partial` outcome rather than hiding it. | FR-006-AC-5 |
| FND-606 | low | ACCEPTED. No gate can detect a rewritten `scripts/assurance_chain.py`; a driver cannot police its own replacement. Git history and pull-request review are the boundary, which is what CONTRIBUTING.md has always said. Stated rather than implied. | NFR-003 |
| FND-607 | low | DEFERRED to agent-ix/quire-contract-ir#21. `quire coverage --strict` reports `status-column-matches-nothing` and exits 0, so a mis-statused row is invisible. A fabricated row is still caught. The 72/72 figure is a backing figure, not a status-verified one. | TM-001 |

## Mutation probes

Recorded with first result and current result, because a probe table that only
shows the final state is a table written after the fix.

| # | Probe | First | Current |
|---|---|---|---|
| A1 | every rule-conformance row rewritten to `fail` | detected (exit 2) | detected |
| A2 | rule-conformance emptied | detected (exit 2) | detected |
| A3 | counterexample stream deleted | detected (exit 2) | detected |
| A4 | counterexample traces blanked | detected (exit 1) | detected |
| A5 | `witnessReplayed` set false | detected (exit 1) | detected |
| A6 | counterexample verdict pair made to agree | detected (exit 1) | detected |
| A7 | Quire export replaced with `{}` | detected (exit 1) | detected |
| A8 | bounded-evidence limitation stripped from every row | detected (exit 1) | detected |
| A9 | excluded-rule rows removed | detected (exit 1) | detected |
| A10 | malformed row removed | detected (exit 1) | detected |
| A11 | MSRV `build-finished` success flipped to false | detected (exit 1) | detected |
| A12 | compatibility census `matched=false` | detected (exit 1) | detected |
| A13 | five non-conclusive reasons collapsed into one | detected (exit 1) | detected |
| A14 | a provenance row rewritten to `fail` | detected (exit 1) | detected |
| A15 | an outcome word the driver does not name | detected (exit 2) | detected |
| B1 | driver made to run `cargo build` | detected (exit 101) | detected |
| B2 | `require_inputs` fabricates a missing producer output | **NOT DETECTED** — the test asserted only that no producer ran, not that the driver left its inputs alone | detected (exit 101) after FND-503 added the input-digest stability assertion |
| B3 | producer shims not written at all | **NOT DETECTED** — stale shims from a previous run were still on disk | detected (exit 101) after FND-502 made `producer_shims` clear its directory |
| B4 | `quoin` replaced by a canned success the driver never runs | detected (exit 101) | detected |
| C1 | `test` deleted from the `ci` prerequisite list | not applicable — the assertion did not exist yet; the probe and FND-504's fix were written together. With that assertion removed, `grep -rn 'starts_with("ci: ")' tests scripts` returns nothing else, so nothing in the repository read the list. | detected (exit 101) |
| C2 | `assurance` deleted from the `ci` prerequisite list | as C1 | detected (exit 101) |
| C3 | a deleted evidence verifier reintroduced | detected (exit 101) | detected |
| C4 | the frozen manifest schema edited | detected (exit 101) | detected |
| D1..D6 | the compatibility view's six built-in probes: collapse non-success states, repair an unreadable outcome, accept a refused schema, unbind the tamper digest, drop the source identity, merge record identities | 6/6 detected | 6/6 detected |

Two probes that were run and are reported as *bad probes* rather than as
detections or as gate failures: an early B2 injected `quire coverage` into the
driver without redirecting its output, which breaks no stated property; and an
early B4 removed only the `shutil.which` pre-flight, leaving the real `quoin`
invocation intact. Both were replaced with mutations that actually break the
property.

## Empty-set audit

| Collection | Site | Empty behaviour |
|---|---|---|
| producer rows | `assurance_chain.py::_rows_result` | raises `ChainError`: "a producer that reported nothing is vacuous, and vacuous is not passed" |
| producer file bytes | `assurance_chain.py::derive_result` | raises on an empty file before any parsing |
| `INPUTS` | `assurance_chain.py::require_inputs` | a literal seven-entry table; an absent file raises and names `make assurance-inputs` |
| `RESULT_PRECEDENCE` match | `assurance_chain.py::_worst` | raises rather than returning a default |
| `mismatch_rows` | chain scenario `counterexamples-are-retained-not-counted` | guarded by `declared > 0`, so an empty producer stream cannot satisfy it vacuously |
| `equivalent_rows` | chain scenario `bounded-equivalence-is-never-generalized` | guarded by `bool(equivalent_rows)` |
| `unsupported_rows` | chain scenario `excluded-rules-stay-unsupported` | guarded by `declared_rules["excluded"] > 0` |
| `malformed_rows` | chain scenario `malformed-input-is-named-not-answered` | guarded by `declared > 0` |
| adapter stream lines | `assurance_chain.py::adapt_conformance` | raises on an empty stream; probe 6 exercises it |
| corpus cases | both producers | exit 2 on an empty `cases` list |
| corpus counts | `counterexample_evidence.rs`, `declared_counterexample_counts` | the declared counts must equal the listed cases in both readers, or it is an environment error rather than a failing row |
| catalog vs corpus id sets | `rule_conformance.rs` | exact set equality in both directions; exit 2 on any difference |
| checksum manifest lines | `check_provenance.py::corpus_rows` | an empty manifest yields a `vacuous` row, which is not `pass` |
| oracle cases | `check_provenance.py::oracle_rows` | an empty `cases` list yields a `vacuous` row |
| provenance entries | `check_provenance.py::build_report` | raises: "no provenance obligation was evaluated at all" |
| compatibility cases | `legacy_evidence_view.py::census` | `accepted_positive_controls` must be non-empty, so a census that only ever saw refusals cannot match |
| retained envelopes | `legacy_evidence_view.py::retained_envelopes` | raises when the glob is empty |
| required states/outcomes | `legacy_evidence_view.py::census` | literal tuples; any undemonstrated member fails the census |
| controls | `assurance_chain.py::run_chain` | a control naming a non-existent scenario raises with exit 2; TC-027 exercises it |
| `ci` prerequisites | TC-029 | exact set equality against a literal thirteen |

## What this analysis did not establish

- Hosted CI was not dispatched, by program rule. The `rust` job's payload is
  `make ci`, which was run locally; the `msrv` job's command was run locally;
  the `licenses` job's `cargo deny` was run locally.
- No `ix-flow` decision event was created, so the receipt path was exercised
  only in its `incomplete` form.
- Nothing here establishes that the rewrite rules are sound. That rests on the
  bounded evidence FR-004 already owned and on the six independent evaluators
  the v0.1 review rounds wrote; this migration transcribes those results and
  does not re-derive them.
