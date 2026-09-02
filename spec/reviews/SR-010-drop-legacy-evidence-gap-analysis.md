---
id: SR-010
title: Gap analysis of the legacy evidence deletion
type: SpecReview
analysis: gap-analysis
scope: spec/**/*.md, assurance/**/*, scripts/**/*, tests/**/*.rs, Makefile
review_set: all
---

# Gap analysis of the legacy evidence deletion

## Summary

Every matrix row is backed. `quire coverage --scope . --strict` reports **68 of
68** with no unbacked row and no status lie, against **72 of 72** before the
deletion. `make ci` exits 0 at the reviewed head with 38 Rust tests.

The gate this pass had to satisfy is *no row becomes unbacked as a side effect* —
not "zero unbacked rows", which was already true before and would have been
satisfied by a change that deleted a row and its test together while quietly
orphaning a third. Both exports are captured and the arithmetic is shown below.

An independent adversarial review re-derived every headline figure in this
document from the tree, and rebuilt the per-outcome state census below from
`assurance_chain.py` rather than trusting the table. Both matched. It also
confirmed `NFR-002-AC-2`, which the deleted `TC-026` also carried, is still
backed by `TC-016` and `TC-030`. It returned nine findings against the change,
including two highs this pass missed; they are recorded in `SR-009`.

## What 72/72 and 68/68 do and do not mean

Carried forward from `SR-008` unchanged, because it is still true and dropping it
would overstate this result.

**Both figures are backing figures, not status-verified ones.** `quire coverage`
reports `status-column-matches-nothing` for the Functional and Stakeholder
tables: the configured status column and the archetype's asserted columns are
mutually exclusive in a validated repository, so status classification is
skipped, and complete-but-unbacked rows could not be checked for those tables.
That is `agent-ix/quire-contract-ir#21`, and it is unchanged by this work.
`68/68` means every row names a backing symbol that exists. It does not mean
every row's declared status was verified against its evidence.

The `status_lies` branch in `derive_result` is kept for the same reason it was
kept before: it does fire for the tables Quire can classify.

## Coverage arithmetic, before and after

Captured with `quire coverage --scope . --json` at `b1bb55a` and at the reviewed
head.

| Document | Before | After | Δ |
|---|---|---|---|
| `spec/evidence/suites.md` | 7/7 | 7/7 | — |
| `spec/requirements/FR-001-rule-catalog.md` | 3/3 | 3/3 | — |
| `spec/requirements/FR-002-bounded-engine.md` | 3/3 | 3/3 | — |
| `spec/requirements/FR-003-rewrite-traces.md` | 3/3 | 3/3 | — |
| `spec/requirements/FR-004-equivalence.md` | 3/3 | 3/3 | — |
| `spec/requirements/FR-005-interchange-evidence.md` | 3/3 | 2/2 | −1 |
| `spec/requirements/FR-006-shared-assurance-intake.md` | 7/7 | 6/6 | −1 |
| `spec/requirements/NFR-001-determinism-resources.md` | 3/3 | 3/3 | — |
| `spec/requirements/NFR-002-provenance-boundary.md` | 2/2 | 2/2 | — |
| `spec/requirements/NFR-003-qualification-integrity.md` | 5/5 | 4/4 | −1 |
| `spec/requirements/StR-001-reviewable-rewrites.md` | 2/2 | 2/2 | — |
| `spec/requirements/StR-002-bounded-equivalence.md` | 2/2 | 2/2 | — |
| `spec/test-matrix.md` | 29/29 | 28/28 | −1 |
| **Total** | **72/72** | **68/68** | **−4** |

`unbacked_rows` is `[]` in both exports. Nine of the thirteen groups are
untouched. The four removed rows are exactly the four claims about material that
no longer exists:

| Row | Why it went |
|---|---|
| `FR-005-AC-2` | asserted every byte of the eleven retained records is unchanged |
| `FR-006-AC-4` | asserted retained bytes are read through the mapping unmodified |
| `NFR-003-AC-4` | asserted retained evidence is read without a byte moving |
| `TC-026` | the test that backed all three |

Each was checked for an alternative discharge before being deleted rather than
weakened. All three criteria have the deleted archive as their grammatical
subject; no surviving test could discharge them, so they were removed. That check
did change one outcome elsewhere — see the state-census section below and
`FND-905` in `SR-009` — where a criterion **was** already discharged and was
therefore not weakened.

`FR-006-AC-7` was the one criterion amended rather than removed. Its core claim
(no local generic evidence framework remains) survives and is still backed by
`TC-029`; the trailing clause about the two retained schemas being *"retained
under their measured status"* was removed because those schemas are gone, and the
criterion was widened to name the archive and the view.

## State census: which source supplied each verification outcome

This is the measurement that matters most, and it was taken **from the
pre-deletion tree**, before anything was removed. A sibling repository measured
the same thing and found its compatibility census was the sole source of
`malformed`: deleting it would have left a twelve-outcome test passing at eleven,
which is a silently weakened gate rather than a red one.

`report["states_demonstrated"]` is built by `assurance_chain.py` from
`chain["scenarios"]` and the adapter probes only. The pre-deletion union in
`TC-027` added the census's `mapped_states` on top. Measured per outcome at
`b1bb55a`:

| Outcome | Demonstrators from the chain | Supplied only by the census? |
|---|---|---|
| `pass` | 3 — `retain-producer-output`, `re-verify-the-sealed-receipt`, `accepts-the-real-run` | no |
| `fail` | 2 — `counterexamples-are-retained-not-counted`, `attested-failed` | no |
| `unavailable` | 1 — `attested-unavailable` | no |
| `unsupported` | 4 — `excluded-rules-stay-unsupported` and three adapter probes | no |
| `inconclusive` | 2 — `non-conclusive-reasons-stay-distinct`, `declared-unknowns-are-carried-not-dropped` | no |
| `not-computed` | 3 — `attested-not_computed`, `audited-clean-versus-unaudited`, `adapter-preserves-non-success-outcomes` | no |
| `malformed` | 1 — `malformed-input-is-named-not-answered` | no |
| `partial` | 2 — `receipt-reports-the-absent-human-decision`, `unattested-proof-stays-missing` | no |
| `stale` | 1 — `stale-candidate-binding` | no |
| `suspect` | 1 — `audit-reports-a-suspect-link` | no |
| `vacuous` | 2 — `audit-reports-a-vacuous-run`, `refuses-an-empty-stream` | no |
| `tampered` | 2 — `refuse-an-edited-receipt`, `retained-bytes-changed-after-sealing` | no |

**Zero of the twelve were reachable only through the compatibility census.** The
census's own six-word legacy vocabulary — `passed`, `failed`, `error`, `skipped`,
`inconclusive`, `unavailable` — intersected the required twelve at only
`inconclusive` and `unavailable`, and the chain owns both. The four
single-demonstrator outcomes were each traced to their source in
`assurance_chain.py`: `attested-unavailable` and `stale-candidate-binding` derive
from `PROOF-rule-conformance` and the receipt path, `malformed-input-is-named-not-answered`
from the counterexample corpus, and `audit-reports-a-suspect-link` from the audit
probes. None reads `legacy-compatibility.json`.

Confirmed after the deletion: the chain reports **12** states demonstrated,
`matched: true`, and `TC-027` passes. `TC-027` no longer unions in a second
source, and it now asserts `demonstrated.len() == REQUIRED.len()` so that the set
must be seen to be complete rather than merely to contain the twelve — a chain
that shrank would fail rather than pass quietly.

The census also demonstrated all three of its own outcomes (`lossy`,
`incompatible`, `unreadable`) and all six legacy states, with two accepted
positive controls. That vocabulary belonged to the deleted mapping and has no
consumer left; it is removed with its subject, not restated.

## Attested proof obligations: 7 → 6

`PROOF-legacy-compatibility` is removed with its `INPUTS` entry, its
`derive_result` branch and its declaration entry. The remaining six —
`PROOF-rule-conformance`, `PROOF-counterexample-evidence`,
`PROOF-normalization-sweep`, `PROOF-provenance-integrity`,
`PROOF-quire-static-export`, `PROOF-msrv` — all attest `passed` at the reviewed
head, and `TC-024` asserts the count is 6 with the reason for the change written
into the assertion message rather than left as a bare number.

## Empty-set audit

Every derived collection this change touched, and what happens when it is empty.

| Collection | Guard | Status |
|---|---|---|
| `consumed_artifacts` digests | `digested == 0` reports a vacuity | ✓ populated (1), and the guard is now probed directly |
| `frozen_schemas` | had its own empty guard | removed with its subject rather than kept over an empty block |
| `states_demonstrated` | `TC-027` requires exactly 12 | ✓ 12, and now an equality rather than a containment |
| `attested_results` | `TC-024` requires exactly 6 | ✓ 6 |
| chain `scenarios` / `controls` / `adapter_probes` | `matched` requires all three non-empty | ✓ unchanged |
| `unbacked_rows` | `derive_result` fails the proof on any entry | ✓ empty |
| TC-029 source census | `inspected > 60` | ✓ still holds after `schemas/` left the walk |
| TC-019 document set | six required tokens over four documents | ✓ all six present; see `FND-905` |
| TC-030 provenance entries | `entries.len() >= 9` | ✓ unchanged |

## What this cleanup got wrong, three times running

Worth recording as a rule rather than as three separate findings, because it is
about remediation work in general and not about this repository.

> **Each fix was checked for what it started catching and not for what it stopped
> catching.**

Three consecutive rounds, each a regression introduced by the previous round's
fix, and none of them caught by the gate the fix had just strengthened:

1. **The extension allow-list dropped the `Makefile`.** Moving the census from a
   three-file root list to a filtered walk closed the hole where `CLAUDE.md` was
   invisible — and silently stopped scanning the one file where `compat-view`,
   `COMPAT_RESULT` and the `assurance-inputs` recipe line would reappear.
   `Path::new("Makefile").extension()` is `None`.
2. **`git ls-files` dropped untracked files.** Rebuilding the census on Git
   closed the hole where deleting a directory from an array removed it from the
   walk *and* from the guard meant to notice — and stopped scanning untracked
   files, which is exactly the state a reintroduced reader is in while someone is
   still writing it.
3. **The control verified Git rather than the census.** Adding a positive control
   for that untracked half closed the hole where the property had no guard at all
   — and asserted that a *fresh* `git ls-files --others` call could see a file it
   had just written, which is true whether or not the census uses the result.
   Deleting the scan loop left it green.

Each fix was probed, and each probe passed, because each probe tested the
property the fix had added. None tested the property the fix had removed. The
question that would have caught all three is not *"does the new check fire?"* but
**"what did the old code see that the new code does not?"**

A related habit is worth naming with it. When the independent reviewer's
`--directory` probe came back green against fix 3, the author's first reading was
that the probe was imprecise. It was not: a control inside a *tracked* directory
survives `--directory`, while a reader dropped into a brand-new directory gets
collapsed and missed. Re-testing rather than explaining away is what found it.

## Findings

Residual gaps, each DEFERRED to a linked issue or ACCEPTED with a rationale. The
implementation findings this pass raised and fixed are in `SR-009`.

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-1001 | medium | DEFERRED to `agent-ix/tl-rewrite#11`. The Make execution-control residue is unchanged in kind and smaller in size: 25 ignored recipe failures where 28 were recorded, because three sabotaged recipe lines went with the deletion. Re-measured with the same named command rather than carried forward. The guard is not re-added, by owner decision. | NFR-003-AC-1 |
| FND-1002 | medium | ACCEPTED. The digest-pinned consumed-artifact population fell from four to one. The compatibility-view schema and both PGM-01 positive-control fixtures are no longer re-derived by anything here, because nothing here reads them. The surviving pin is live and probed red; the reduction is real and is stated rather than implied. | FR-006-AC-1 |
| FND-1003 | medium | ACCEPTED. Mutation probes fell from 46 to 36. Twelve died with the material they guarded and two were added; the base of 46 is `SR-008`'s 41 rows with its `D1..D6` row counted as the six probes it is. The count fell because there is less to probe, not because anything was fixed; the per-probe accounting is in `SR-009`. | NFR-003-AC-3 |
| FND-1004 | low | DEFERRED to `agent-ix/engineering-assurance#21`, dispositioned `accepted` and not `resolved` in the change-assurance record. The pinned mapping still has no reader for `quire.derivation-evidence/v1`. This repository no longer holds a subject the gap applies to; the gap itself is unfixed. | FR-006-AC-1 |
| FND-1005 | low | DEFERRED to `agent-ix/quire-contract-ir#21`. `68/68` is a backing figure, not a status-verified one: `quire coverage` skips status classification for the Functional and Stakeholder tables. The caveat is carried forward from `SR-008` unchanged rather than dropped now that the number is smaller. | FR-006-AC-3 |
| FND-1008 | medium | DEFERRED to `agent-ix/tl-rewrite#15`, and **discovered by this pass rather than introduced by it**. `a_control_naming_a_scenario_that_does_not_exist_is_refused` (TC-027) fails intermittently — measured **2 in 8** with the test run in isolation — because the probe symlinks `target` into its own scratch directory, so `STORE` resolves back through `target/dangling-probe/target` to the real `target/assurance-store` and the probe shares its Quoin store rather than owning one. Quoin then writes a non-JSON warning to stdout when its working directory has gone, and `receipt()` parses stdout as JSON. It produces a false **red**, not a false green, but a gate that fails a quarter of the time for reasons unrelated to its property trains readers to re-run instead of read, and `make ci` inherits the rate. Introduced with the probe in issue #9, not here. Explicitly ruled out as an effect of this change's new untracked-file control: the rate is unchanged with that test filtered out, and the control was moved under `scripts/` so it is not symlinked into the scratch at all. | NFR-003-AC-3 |
| FND-1007 | medium | ACCEPTED, with the gap named. Two of the nine findings the independent review raised were false statements left standing in `assurance/change-assurance.json` after being corrected everywhere else — and that file is the one `assurance_chain.py` seals and carries into the verification receipt. `TC-029`'s census exempts it by design, because it is where the deletion is recorded and must name what was deleted. **No gate can catch a false claim in the sealed declaration.** Human review is the only control on that file's prose, which is what caught these two. Recorded here rather than closed with a check that would have to exempt itself. | FR-006-AC-4 |
| FND-1006 | low | ACCEPTED. `agent-ix/tl-mltl#11` is left open and `retain/tl-mltl-v0.1-da2c7704` is left in place. This change removes the last references to `da2c7704` in the campaign, so nothing requires it fetchable, but disposal is a separate step once all eight deletions have landed and is not taken here. | NFR-002-AC-2 |

## Residual and unchanged

- **`agent-ix/tl-rewrite#11` stays open.** The Make execution-control guard is
  not re-added. Its residue was **re-measured** with the same named command
  rather than carried forward: the sabotage run now exits 0 after **25** ignored
  recipe failures where it recorded 28, because three sabotaged recipe lines went
  with the deletion — `compat-view`'s two and the compatibility view's line
  inside `assurance-inputs` — and `assurance` went from three sub-targets to two.
  Nothing was fixed; there is less to neuter. `make ci` with the same variables
  and no `.IGNORE:` still exits 2 at the first prerequisite, and all 13 `ci`
  prerequisites still report success under `.IGNORE:`.
- **`agent-ix/engineering-assurance#21` stays open** and is dispositioned
  `accepted`, not `resolved`. The upstream mapping still has no reader for
  `quire.derivation-evidence/v1`. What changed is that this repository no longer
  holds a subject the gap applies to.
- **`agent-ix/tl-mltl#11` is not closed here** and
  `retain/tl-mltl-v0.1-da2c7704` is not deleted here. This change removes the
  last references to `da2c7704` anywhere in the campaign — `tl-mltl`'s own
  deletion, merged as `58f858e`, already removed the eight files on its side — so
  nothing now requires that revision fetchable. Disposal is handled separately
  once all eight deletions have landed.
- **Mutation probes fell from 46 to 36.** Twelve died with the material they
  guarded and two were added. The accounting is in `SR-009`.
- **The digest-pinned artifact population fell from four to one.** Stated rather
  than implied: the compatibility-view schema and both PGM-01 positive-control
  fixtures are no longer re-derived by anything here.
- No pin was changed. No tag, publish, or release was made. Hosted CI was not
  dispatched.
