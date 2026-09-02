---
id: SR-008
title: Closing gap analysis of the tl-rewrite shared assurance migration
type: SpecReview
analysis: gap-analysis
scope: spec/**/*.md, assurance/**/*, scripts/**/*, examples/**/*.rs, tests/**/*.rs, corpus/**/*, evidence/**/*, Makefile, .github/workflows/ci.yml
review_set: all
---

# Closing gap analysis of the tl-rewrite shared assurance migration

## Summary

Every matrix row is backed: `quire coverage --scope . --strict` reports 72 of 72
over 39 evidence symbols with no status lies and no unbacked rows. `make ci`
exits 0 at the exact head on a clean tree, with 39 Rust tests.

This closing pass records the mutation-probe table in full, with first and
current results, because a probe table that shows only the final state is a
table written after the fix. Four probes were initially not detected. Three were
real, and one was a bad probe.

## Findings

Residual gaps, each DEFERRED to a linked issue or ACCEPTED with a rationale.

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-801 | medium | DEFERRED to `agent-ix/tl-rewrite#11`. The Make execution-control guard is gone. With the exact command recorded in four places, `.IGNORE:` takes `make ci` from exit 2 to exit 0 after 28 ignored recipe failures, with all 13 `ci` prerequisites reporting success. `assurance-chain` is among the recipes that fail and are ignored — the chain *does* detect the injected defect and returns non-zero, and Make discards it, which is the sharpest statement of what was lost. | NFR-003 |
| FND-802 | medium | DEFERRED to `agent-ix/engineering-assurance#21`. The pinned mapping has no reader for `quire.derivation-evidence/v1`, so all 11 retained envelopes are refused. Reported as the compatibility result. | FR-006-AC-4 |
| FND-803 | medium | DEFERRED to `agent-ix/quire-contract-ir#21`. `quire coverage --strict` cannot classify the status column for the Functional and Stakeholder tables, so a mis-statused row is invisible. `unbacked_rows` is the field that does work and is now gated in two places. | TM-001 |
| FND-804 | low | DEFERRED to `agent-ix/engineering-assurance#20`. The tagged release records `pending_human_acceptance` and ships no predicate. Reported, never gated on in either direction. | FR-006-AC-1 |
| FND-805 | low | DEFERRED to `agent-ix/tl-mltl#11`. The eleven retained records name `tl-mltl` `da2c7704`, which was reachable from no remote ref until `origin/retain/tl-mltl-v0.1-da2c7704` was pushed. That branch is what keeps the revision those immutable records declare fetchable and must not be deleted. This repository no longer pins it. | NFR-002-AC-2 |
| FND-806 | low | ACCEPTED. No `ix-flow` decision event exists, so the receipt reads `incomplete` with a missing decision. That is the correct state for an unreviewed change; only the repository owner may create one, and the chain demonstrates it as the `partial` outcome rather than hiding it. | FR-006-AC-5 |
| FND-807 | low | ACCEPTED. No gate detects a rewritten `scripts/assurance_chain.py`. A driver cannot police its own replacement; Git history and pull-request review are the boundary. | NFR-003 |

## Dual run, recorded as observed

| Path | Revision | Result |
|---|---|---|
| old — base Makefile plus the local evidence scripts | `74aa3f1` (pristine base) | **`make ci` exit 2.** Eleven of twelve prerequisites pass; `verify-evidence` fails with `assured evidence is not v2-qualified`. 29 Rust tests pass. |
| old — same, at the coexistence revision | `473bd97` | **exit 2** at `check-failure-propagation`. Individually: `verify_evidence.sh` 1, `run_policy_tests.py` 1, `tool_identity.py --verify-live` 1, `check_traceability_coverage.py` 0 (72/72). |
| new — at the coexistence revision | `473bd97` | **exit 2** on one assertion only: TC-029, which requires the local framework to be gone. 36 of 37 Rust tests pass. |
| new — after the deletion commit | `a65f1f3` onward | **exit 0.** 39 Rust tests pass. |

The base was not green, and the reason is one this migration deletes rather than
repairs: the head's own `evidence_profile.resolve_profile` requires the active
retained record's source revision to carry machinery it does not have. Round 12
of PR #8 predicted it by reading; this confirms it by execution. **No green
baseline was manufactured.**

## Mutation probes

29 in the self-review, 37 in the adversarial review, plus 11 re-probes after the
fixes. Recorded with first and current results.

### Producer degradation, read by the chain

| # | Probe | First | Current |
|---|---|---|---|
| A1 | every rule row rewritten to `fail` | detected (2) | detected |
| A2 | rule stream emptied | detected (2) | detected |
| A3 | counterexample stream deleted | detected (2) | detected |
| A4 | counterexample traces blanked | detected (1) | detected |
| A5 | `witnessReplayed` set false | detected (1) | detected |
| A6 | counterexample verdict pair made to agree | detected (1) | detected |
| A7 | Quire export replaced with `{}` | detected (1) | detected |
| A7b | Quire export replaced with `{"junk":[1]}` | detected (1) | detected |
| A8 | bounded-evidence limitation stripped | detected (1) | detected |
| A9 | excluded-rule rows removed | detected (1) | detected |
| A10 | malformed row removed | detected (1) | detected |
| A11 | MSRV `build-finished.success` flipped | detected (1) | detected |
| A12 | compatibility census `matched:false` | detected (1) | detected |
| A13 | five non-conclusive reasons collapsed | detected (1) | detected |
| A14 | a provenance row rewritten to `fail` | detected (1) | detected |
| A15 | an outcome word the driver does not name | detected (2) | detected |
| A16 | all provenance rows → `malformed` | **NOT DETECTED — exit 0** | detected, FND-701 |
| A17 | all normalization rows → `malformed` | **NOT DETECTED — exit 0** | detected, FND-701 |
| A18 | a real garbage line in `corpus/west-v1/SHA256SUMS`, end to end | **NOT DETECTED — producer exit 1, chain exit 0, `passed`** | detected, chain exit 1 |
| A19 | absent / empty / unreadable input | detected (2) each | detected |
| A20 | all five bounded-domain cases removed from corpus and producer | **NOT DETECTED — scenario `ok`, `inconclusive` still "demonstrated"** | detected, FND-705 |

### The producer boundary

| # | Probe | First | Current |
|---|---|---|---|
| B1 | driver made to run `cargo build` | detected (101) | detected |
| B2 | driver fabricates a missing producer output | **NOT DETECTED** — the test asserted only that no producer ran | detected (101) |
| B3 | producer shims not written at all | **NOT DETECTED** — stale shims from an earlier run were still on disk | detected (101) |
| B3b | shims removed entirely, after the B3 fix | **NOT DETECTED — exit 0** | detected (101), FND-702 |
| B4 | `quoin` replaced by a canned success | detected (101) | detected |
| B5 | driver runs `quire coverage`, output discarded | **NOT DETECTED** | detected (101), FND-704 |
| B6 | driver runs `python3 scripts/check_provenance.py` | not probed before FND-704 | detected (101) |
| B7 | driver runs `.venv-assurance/bin/python scripts/legacy_evidence_view.py` | not probed before FND-704 | detected (101) |
| B8 | driver regenerates a deleted input, after FND-704 | not probed before | detected (101) |

### Gate structure and the retained archive

| # | Probe | First | Current |
|---|---|---|---|
| C1 | `test` deleted from `ci:` | assertion did not exist; nothing else read the list | detected (101) |
| C2 | `assurance` deleted from `ci:` | as C1 | detected (101) |
| C3 | a deleted evidence verifier reintroduced | detected (101) | detected |
| C4 | the frozen manifest schema edited | detected (101) | detected |
| C5 | a matrix row repointed at nonexistent test cases | **NOT DETECTED — totals stay 72/72** | detected, FND-703 |
| C6 | one retained record removed from `evidence/` | detected, but by an uncaught traceback | detected by a named refusal, FND-718 |
| C7 | a compatibility fixture's free-text `kind` relabelled | no effect — confirms the twelve-state census counts measured outcomes, not labels | unchanged |
| C8 | `--mutation-probes` with an empty probe set | **NOT DETECTED — `0/0`, exit 0** | detected (2), FND-707 |
| C9 | `consumed_artifacts` digests removed | **NOT DETECTED** | detected (1), FND-708 |
| C10 | classification list emptied | **NOT DETECTED** | detected (1), FND-708 |
| D1..D6 | the compatibility view's six built-in probes | 6/6 detected | 6/6 detected |

Two probes are reported as **bad probes** rather than as detections or gate
failures: an early form of B5 injected `quire coverage` without redirecting its
output, which at that time broke no stated property; and an early B4 removed only
the `shutil.which` pre-flight, leaving the real invocation intact. Both were
replaced with mutations that break the property, and B5's real form then found
FND-704.

## Empty-set audit

Every derived collection the new code iterates, and what happens when it is
empty. Entries marked ✗ were found by the adversarial review and are now fixed.

| Collection | Site | Empty behaviour |
|---|---|---|
| adapter stream lines | `adapt_conformance` | raises ✓ |
| adapter protocol table | `adapt_conformance` | an unnamed protocol raises ✓ |
| producer rows | `_rows_result` | raises: "vacuous is not passed" ✓ |
| per-proof outcome table | `_rows_result` | a proof with no declared vocabulary raises ✓ |
| producer file bytes | `derive_result` | raises on an empty file ✓ |
| `INPUTS` | `require_inputs` | literal seven-entry table; an absent file raises and names the target ✓ |
| precedence match | `_worst` | raises ✓ |
| `declared_rule_counts` | chain | guarded by `excluded > 0` ✓ |
| `declared_counterexample_counts` | chain | self-consistency checked; the four counts are pinned in TC-028 ✓ |
| `mismatch_rows` | chain scenarios | guarded by `declared > 0` and `bool(...)` ✓ |
| `equivalent_rows` | chain scenario | guarded by `bool(...)` ✓ |
| non-conclusive reasons | chain scenario | ✗ `[] == []` → now guarded by `> 0` |
| `EXECUTED` | `command_audit` | the scenario requires `bool(EXECUTED)` and a control requires more than five ✓ |
| scenarios / controls / probes | `main` | ✗ `all([])` was `True` → now requires all three non-empty |
| `msrv` finished messages | `derive_result` | → `not_computed` ✓ |
| `unbacked_rows` | `derive_result` | present and non-empty → `failed` ✓ |
| audit findings | adapter probes | probes require `> 0` ✓ |
| compatibility cases | `census` | a positive control must be accepted ✓ |
| retained envelopes | `retained_envelopes` | raises ✓ |
| required states / outcomes | `census` | literal tuples; any undemonstrated member fails ✓ |
| mutation probes | `run_mutation_probes` | ✗ `0/0` was `True` → now the declared set is required |
| `consumed_artifacts` digests | `check_shared_pins` | ✗ clean over nothing → now an explicit problem |
| `classify_all` result | `check_shared_pins` | ✗ `accepted([])` was `True` → now requires a non-empty list |
| `frozen_schemas` digests | `check_shared_pins` | already guarded ✓ |
| checksum manifest lines | `check_provenance` | an empty manifest is a `vacuous` row, not a pass ✓ |
| oracle cases | `check_provenance` | an empty `cases` list is a `vacuous` row ✓ |
| provenance entries | `check_provenance` | raises ✓ |
| catalog vs corpus ids | `rule_conformance.rs` | exact set equality both ways, exit 2 on any difference ✓ |
| `ci` prerequisites | TC-029 | exact set equality against a thirteen-element literal ✓ |
| source census | TC-029 | `inspected > 60` asserted ✓ |
| producer inputs | TC-024 | `inputs_before` required non-empty ✓ |

## What this analysis did not establish

- Hosted CI was not dispatched, by program rule. The `rust` job's payload is
  `make ci`, run locally; the `msrv` job's command was run locally; the
  `licenses` job's `cargo deny` was run locally.
- No `ix-flow` decision event was created, so the receipt path was exercised
  only in its `incomplete` form.
- Nothing here establishes that the rewrite rules are sound. That rests on the
  bounded evidence FR-004 already owned and on the six independently written
  evaluators of the v0.1 review rounds; this migration transcribes those results
  and does not re-derive them.
- Deleting an individual `assert!` inside a test is detected by nothing, as
  expected.
