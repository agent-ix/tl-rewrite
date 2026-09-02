---
id: SR-009
title: Code review of the legacy evidence deletion
type: SpecReview
analysis: code-review
scope: assurance/**/*, scripts/**/*, tests/**/*.rs, spec/**/*.md, Makefile, CLAUDE.md
review_set: all
---

# Code review of the legacy evidence deletion

## Summary

The change deletes 594 tracked files, 14,695 lines: the retained evidence
archive, its only reader, the fixtures and schemas that served it, and the chain
proof obligation that read it. It is authorised by `agent-ix/engineering-assurance#7`, section
*"Preservation constraint released for the pre-stable phase"*, an owner decision
of 2026-09-02 transcribed by an agent. The epic's completion criterion and its
mandatory control were amended before this work, so no live constraint was
violated.

The deletion is irreversible from the working tree, so the only question worth
real scrutiny is whether anything still needs what was removed. Three classes
were checked deliberately rather than assumed: whether each schema was actually
dead, whether any verification outcome was reachable only through the deleted
census, and whether any check was left asserting over an empty population.

`make ci` exits 0 at the reviewed head, with 38 Rust tests.
`quire coverage --scope . --strict` reports 68 of 68 rows backed and no unbacked
row, against 72 of 72 before.

**25 findings: 5 high, 11 medium, 9 low.** Nine came from an independent
adversarial review commissioned after this document's first draft, including
both of the highs below that this self-review missed; they are recorded in their
own section, and three of them correct claims this document itself made.

## Authority and boundary

- The deletion is permitted. **Rewriting is not, and none was done.** No record
  was backdated, re-sealed, moved, or edited to look like it still verifies.
- The constraint re-applies unchanged at the move toward stable releases.
- `agent-ix/tl-rewrite#11` is untouched: the Make execution-control guard is
  **not** re-added, and the `ci:` prerequisite set is unchanged at 13.
- Hosted CI was not dispatched. It remains `workflow_dispatch`-only.
- The `Cargo.toml` pins are unchanged: `tl-syntax` at `953ee825`, `tl-mltl` at
  `f7eb8bdf`. Nothing was repinned.

## What was deleted, measured in this tree

| Item | Measured |
|---|---|
| `evidence/` | 582 tracked files, 3,720,647 bytes, 11 records |
| Retained envelope family | `quire.derivation-evidence/v1`, all 11 |
| Mapping outcome for all 11 | `incompatible`, *"unknown PGM-01 schema version"* |
| True PGM-01 records held | **0** |
| `scripts/legacy_evidence_view.py` | 507 lines, the only reader |
| `tests/fixtures/legacy-compat/` | 8 files including `expectations.json` |
| `schemas/` | 3 files |
| Total | **594 files, 14,695 deleted lines** |

This repository is **not** `quire-contract-ir`. It held no true PGM-01 record.
The refusal figure was measured against its own bytes, not inherited: `tl-syntax`
reported the identical refusal over 23 envelopes and `tl-parse` over 12, while
`quire-contract-ir` retained real PGM-01 records that mapped and
`quire-analyze`'s Markdown summaries read as `unreadable` rather than
`incompatible`.

## Both schemas were proved dead before deletion

Two sibling repositories hold schemas that look frozen by name and are live
output contracts — `quire-contract-codegen`'s
`pgm01-derivation-evidence-envelope-v1.schema.json` is included as `PGM_SCHEMA`
and validated on every generation, and `quire-analyze`'s
`differential-report-v1.schema.json` is included by `src/report.rs` and validated
on every published document. Both were kept in those repositories. No freeze list
was inherited here.

| Evidence gathered | Result |
|---|---|
| `include_str!` / `include_bytes!` anywhere in the tree | **0 hits** |
| `import jsonschema` / `from jsonschema` anywhere | **0 hits** |
| Either schema filename in `src/` | **0 hits** |
| Either schema filename in `scripts/` | **0 hits** |
| Either schema filename in `tests/` | 2 hits, both in the test that pinned their digests |
| Either schema filename in `assurance/` | 3 hits, all in the freeze declaration itself |

Nothing validated any document against either file. `schemas/README.md` had
already recorded that `make ci` ran no JSON-Schema validation and that no script
imported `jsonschema`. Both files existed for exactly one reason — the retained
envelopes named them by digest — and that reason went with the envelopes.

`schemas/README.md` was deleted with the directory it documented.

## The consumed-artifact population was refilled, not emptied

`assurance/pins.json` declared five `consumed_artifacts`, four of them digest
pinned: `verification_semantics.py`, `schemas/pgm01-compatibility-view-v1.schema.json`,
and the two PGM-01 positive-control fixtures. **Every one of the four was read by
exactly one caller — the deleted compatibility view.** The fifth,
`compatibility-matrix.json`, deliberately carries no digest, because pinning its
bytes would make this repository a second acceptance authority.

Leaving the four would have claimed this repository still consumes artifacts
nothing opens. Removing them without replacement would have left
`artifact_digest_mismatches` re-hashing an empty list. That function's
`digested == 0` branch already existed for exactly that shape and would have
caught it — `make pins` would have gone red with *"nothing to re-derive and a
clean answer would be vacuous"*. A caught vacuity is better than a silent one,
but it is not a check.

The list now pins `engineering_assurance/compatibility.py`
(`62829251…1475f654`), the module `build_report` imports for `accepted`,
`classify_all` and `load_matrix` — every component verdict this repository
reports comes out of it. It is read on every `make pins`, so the deletion cannot
take its subject away.

**Probed, not asserted.** A single newline appended to the installed module:

```
consumed artifact digest mismatch: compatibility.py: b3e8cc8f…b864e994,
  pins record 62829251…1475f654
shared pins NOT accepted
PROBE EXIT=1
```

Restored, exit 0.

**What the fix made vacuous, stated rather than implied:** this repository now
digest-pins exactly one artifact from the release where it pinned four. The
compatibility-view schema and both PGM-01 positive-control fixtures are no longer
re-derived by anything here, because nothing here reads them. That is a genuine
reduction in what `make pins` checks, and it is the honest consequence of
deleting the only caller.

`frozen_schema_mismatches` was removed outright rather than kept over an empty
`frozen_schemas` block, along with its `retained_schema_mismatches` report field
and its contribution to `accepted`.

## Mutation probes: 44 → 34, and why it fell

The migration recorded 46 probe rows in `SR-008`, of which 2 were reported as bad
probes, giving **44 real probes**. Twelve die with the material they guarded:

| Probe | Why it goes |
|---|---|
| A12 compatibility census `matched:false` | the census is deleted |
| B7 driver runs the compatibility view | the program no longer exists to probe |
| C4 the frozen manifest schema edited | the schema is deleted |
| C6 one retained record removed from `evidence/` | the archive is deleted |
| C7 a compatibility fixture's `kind` relabelled | the fixtures are deleted |
| C8 `--mutation-probes` over an empty probe set | the probe runner is deleted |
| D1–D6 the view's six built-in probes | the view is deleted |

Two are added. The `digested == 0` branch of `artifact_digest_mismatches` is now
probed directly, because that branch is what would have caught the deletion
emptying the list, and an unprobed guard is a guard nobody has seen fire. The
per-directory census guard (FND-923) is the second. Both are in the probe table
below, red on the defect each names.

**44 → 32 by deletion, plus 2 added = 34.** The count fell because there is less
to probe, not because anything was fixed. C9 (`consumed_artifacts` digests
removed) and C10 (classification list emptied) survive and C9 now covers the live
pin. C3 (a deleted evidence verifier reintroduced) survives and was widened to
cover this deletion's names as well.

**`compat-view` gated nothing beyond the compatibility view.** Its recipe was two
lines, both invoking `legacy_evidence_view.py`. A sibling found its equivalent
target carrying a broader mutation gate, so this was checked rather than assumed.
Mutation gates still reachable from `make ci` are the chain's five adapter probes
and every control it runs (via `assurance`), and the Rust probes for the dangling
scenario, the stale revision constant, both mirror-scan branches, and the two
consumed-artifact branches (via `test`).

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-901 | high | **FIXED.** `spec/spec.md` sourced the tl-mltl revision identity from retained evidence — *"semantic comparison comes from the exact tl-mltl revision named by retained evidence"*. After the deletion the specification named an authority the repository no longer had, for the identity of the evaluator every `ConformanceReport` attributes verdicts to. Corrected to the resolved dependency graph, which `scripts/check_provenance.py` already enforced and `TC-030` already probes. This is also the exact `da2c7704` linkage: those records named a revision the build had not used since the pin moved. | FR-005-AC-3, NFR-002-AC-2 |
| FND-902 | high | **FIXED.** `spec/assurance/CAC-001.md` required a replacement component to pass *"retained evidence gates"*. There are none, so the clause named a gate no replacement could ever pass — an unsatisfiable condition, not a strict one. Replaced with the gate set that actually runs, `make assurance`, and the substitution is stated in the document rather than made silently. | FR-006-AC-7 |
| FND-903 | high | **FIXED.** `spec/assurance/AA-001.md`'s **Sufficiency Decision** gated the top-level release claim on *"retained evidence names the exact candidate"*, which the deletion made unsatisfiable. Raised by the campaign coordinator from `tl-syntax`, which shipped the same shape. The first replacement was wrong in the opposite direction and is corrected under FND-915; the condition now also requires the receipt not to be `incomplete`, which it currently is. | FR-005-AC-3 |
| FND-904 | medium | **FIXED.** `spec/assurance/MP-001.md`'s collection procedure listed *"canonical PGM-01 validation against one immutable revision"*. That validation was performed against the deleted archive. Removed rather than restated over an empty population, and the removal is named in the document. | FR-006-AC-7 |
| FND-905 | medium | **FIXED.** `tests/wire_records.rs::human_authority_and_qualification_boundaries_remain_open` (TC-019) inspected `evidence/README.md`, which the deletion removed, and that file was the **only** source of the required token `pending` — it said the retained records *"inform a pending human source-release decision"*. Caught by `make ci`, not by reading. The premise was checked before weakening: the property is still true, so rather than dropping the token the statement moved to `AA-001`'s Human Decision section, which is the document that owns the claim. Six properties over four documents, not five over four. | FR-005-AC-3, StR-001-VC-1 |
| FND-906 | medium | **FIXED.** `assurance/pins.json`'s `consumed_artifacts` would have been emptied of every digest-pinned entry. See the section above: replaced with a live pin on the module the surviving caller imports, and probed red with a one-byte upstream edit. Approach taken from `quire-analyze`, which solved the same problem the same way. | FR-006-AC-1 |
| FND-907 | medium | **FIXED.** `"disposition": "moot"` was written into `assurance/change-assurance.json` for the derivation-evidence unknown. Quoin's vocabulary is `open | accepted | deferred | resolved`, and it refused to seal the record — *"unknown disposition"*. Caught by running the chain. Set to `accepted`, which is the honest word: the upstream mapping gap is **not** resolved and nothing here fixed it; what changed is that this repository no longer has a subject it applies to. The entry is kept in the record rather than deleted with the envelopes so a reader can see the gap was accepted and never closed. | FR-006-AC-1 |
| FND-908 | low | **FIXED, and this finding's own first wording was false.** The new docstring in `scripts/check_shared_pins.py` and the new prose in `assurance/README.md` named the deleted reader and obligation by their exact identifiers, and the TC-029 census refused both. The original text of this row also claimed the census refused the same thing in `CLAUDE.md`. It did not and structurally could not — see FND-916. All three were reworded; only two were ever caught. | FR-006-AC-7 |
| FND-909 | low | **FIXED.** `.github/workflows/ci.yml` justified `fetch-depth: 0` on the `rust` job with *"The compatibility view asks Git whether the retained evidence bytes are the committed bytes"* — a dead reason, since the view is deleted and no gate in `make ci` reads Git history any more. Not caught by the TC-029 source census, which matches identifiers and not prose: the comment named the view in words rather than by filename. That is a real limit of the census and is recorded here rather than papered over by adding prose patterns to it. The checkout depth is unchanged and the comment now states the current reason — Git history is the only integrity boundary the deleted bytes have left. | FR-006-AC-7 |
| FND-910 | medium | **FIXED.** The TC-029 source census permitted `spec/reviews/` and `spec/plans/` wholesale so that dated findings records are not rewritten to un-say what they found. That is right for prose and wrong as written: a reintroduced reader dropped into `spec/plans/` as a `.py` or `.rs` file would have been waved through by a rule meant to protect Markdown. Narrowed to `.md` under those two directories. Found by re-reading the exemption rather than by a gate, which is the point of writing exemptions down. | FR-006-AC-7 |
| FND-911 | low | **ACCEPTED.** `scripts/check_provenance.py`'s module docstring still names `da2c7704`. It is not a live requirement on that revision — it is the narrative of the defect the check exists to catch, and `TC-030` restores that exact value in a scratch copy to prove the check goes red. Removing it would remove the reason the check exists. | NFR-003-AC-5 |
| FND-912 | low | **ACCEPTED.** `scripts/assurance_chain.py`'s docstring still says *"Nothing is written under `evidence/`"*. The claim is about the driver's own behaviour — it is not a retention store — and remains true and meaningful now that no such directory exists. | FR-006-AC-2 |

## Findings from the independent adversarial review

An independent agent was commissioned with one instruction — **find what the
author missed** — and reviewed the branch by executing probes rather than
reading. It returned **2 high, 3 medium and 4 low**, verified every headline
number independently, and reproduced the per-outcome state census from
`assurance_chain.py` rather than trusting `SR-010`'s table. Its table matched.

Both highs are the same shape, and it is the shape that matters most here: the
claim was corrected everywhere a reader would look and left standing in the one
file that gets **sealed** and travels into the verification receipt.

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-913 | high | **FIXED.** `assurance/change-assurance.json`'s `UNKNOWN-upstream-pins-are-newly-merged` still asserted, in the present tense, that *"this repository's retained evidence names tl-mltl da2c7704 … that branch is what keeps those records' declared revision fetchable and must not be deleted"*. The identical claim was corrected in `assurance/pins.json` and `AA-001.md`; this copy was missed, and it is the durable one — `assurance_chain.py` seals `record.definition` through `quoin change-assurance seal` and asserts every declared unknown is carried into the receipt. A claim that argued from the retained evidence would have been sealed into the record this change produces. TC-029's census exempts this file by design, so no gate could catch it. Rewritten to match: all eleven records deleted, the last references in the campaign, nothing requires `da2c7704` fetchable, branch and `tl-mltl#11` handled separately. | FR-006-AC-4, NFR-002-AC-2 |
| FND-914 | high | **FIXED.** `assurance/change-assurance.json`'s `UNKNOWN-make-execution-control-guard-removed` carried the pre-deletion figure — *"exit 0 after 28 ignored recipe failures … `assurance` whose three sub-targets each failed"*. The same sentence was re-measured and corrected in `Makefile`, `AA-001.md` and `NFR-003`, and missed in the fourth and only sealed copy. The reviewer re-derived 25 independently with the same named command. Corrected to 25 and two sub-targets, with the delta and its cause stated in the record. | NFR-003-AC-1 |
| FND-915 | medium | **FIXED.** The FND-903 fix replaced an unsatisfiable release condition with one automation satisfies by construction: *"until the sealed change-assurance record names the exact candidate revision"* is true on every `make assurance` run, because the record is bound to `--candidate-revision` unconditionally — a release condition discharged automatically, three lines above a sentence saying automation cannot change the status. That is the mirror of the defect FND-903 fixed. The condition now also requires the receipt not to be `incomplete`, which is falsifiable and is currently **not** satisfied. | FR-005-AC-3 |
| FND-916 | medium | **FIXED.** `FR-006-AC-7` was widened from *"remains in the execution path"* to *"remains in the repository"*, but the TC-029 census walked nine directories plus exactly three named root files, so `CLAUDE.md`, `README.md`, `CONTRIBUTING.md`, `AGENTS.md`, `deny.toml` and the three toolchain configs were never inspected. The reviewer **probed it**: appending the deleted identifiers to `CLAUDE.md` left the test green. The census now discovers every root file rather than listing three, which is also what the deleted `schemas/README.md` had claimed it did and what the code had never done. Re-probed after the fix: exit 101, naming `CLAUDE.md`. | FR-006-AC-7 |
| FND-917 | medium | **FIXED.** `AA-001` claimed the old gate sat *"eleven lines below"* a section disclaiming retained evidence, and `SR-009`'s own FND-903 said *"eleven lines above"* — contradictory, and neither was measured. The reviewer measured the real distances (23 lines at `b1bb55a`, 31 at `ef86a14`) and found no disclaiming section within eleven lines in either direction. The figure was carried over from a sibling's narrative and stated as though measured. Removed from both documents rather than replaced with a corrected number, because the distance was never the point. | FR-005-AC-3 |
| FND-918 | low | **FIXED.** `spec/spec.md`'s Requirements Architecture still allocated *"the versioned interchange and PGM-01 evidence boundary"* to FR-005, whose title, behaviour and `FR-005-AC-2` had all lost the evidence half in the same commit. The allocation is removed rather than reassigned: no requirement owns an evidence boundary this repository no longer has. FR-006 and NFR-003, absent from the sentence before this change, are now named. | FR-005-AC-3 |
| FND-919 | low | **FIXED.** `scripts/assurance_chain.py` and `tests/shared_assurance.rs` both carried comments asserting as current behaviour that repointing a row *"leaves `totals.backed` at 72/72"*, while the adjacent assertions had correctly moved to 68. Both now state the property without a figure that has to be maintained, and name both counts. | FR-006-AC-3 |
| FND-920 | low | **FIXED.** The TC-029 removed-path list asserted both `tests/fixtures/legacy-compat` and `tests/fixtures/legacy-compat/expectations.json` are absent. The second cannot fail unless the first does. Decoration, removed. | FR-006-AC-7 |
| FND-921 | low | **FIXED.** This review's own deleted-line total said **15,108**, which is the `-` column across all 611 changed files in the commit, not the deleted files. The 594 deleted files account for **14,695**; the balance is deletions inside the 17 modified files. Corrected. A figure that counts a different population than its label is the defect this campaign has spent the most effort on. | — |

| FND-922 | medium | **FIXED.** The TC-029 census floor `inspected > 60` was inherited unchanged while its population moved: 90 files before, 92 after, because `schemas/` left the walk and every root file entered it. It would have carried 32 files of slack. Raised by the campaign coordinator from a sibling that shipped a floor whose headroom had silently fallen from 25 to 7. Re-derived to `>= 85` — the population minus the largest directory a routine change could shrink unnoticed — with the derivation written into the assertion, and probed (P2). | FR-006-AC-7 |
| FND-923 | medium | **FIXED.** A total-only floor cannot catch the defect it gestures at. `scripts`, `src` and `corpus` are five files each and `docs` and `.github` are one, so a directory silently dropping out of the walk moves the total by less than ordinary churn and no floor would fire. Every declared directory must now contribute at least one file, and the root walk must contribute at least one. Probed (P1): pointing the walk at a nonexistent path is now red and names the directory. | FR-006-AC-7 |
| FND-924 | low | **FIXED.** The consumed-artifact probe mutated `consumed_artifacts[0]` positionally. Reordering the entries would have written a digest onto the deliberately undigested `compatibility-matrix.json` and caught a file-not-found — a pass for the wrong reason. Now targeted by path, and it asserts the pin exists before mutating it. | FR-006-AC-1 |
| FND-925 | low | **FIXED.** The census root walk pushed `.git`, which is a *file* in a linked worktree holding a `gitdir:` pointer, so the inspected count differed between the main clone and a worktree. Skipped by name; the derived floor is now stable in both. | FR-006-AC-7 |

## Every check this change added, probed

Raised by the campaign coordinator from `quire-contract-runtime`, which found a
real gap left by its deletion, wrote a probe to fill it, and shipped a
**tautology**: the probe rewrote rows in a protocol whose producer cannot emit
the value it tested for, and checked post-adapter where that value collapses
into another. Its census read 12/12 green over a hollowed producer. Filling a
gap with something unfalsifiable reads identically to filling it properly.

The rule applied here: **name the defect the check should catch, introduce that
defect, watch it go red.** Every check this change added or repointed was probed
that way. All five were run; none is reported from reasoning.

| # | Check | Defect introduced | Result |
|---|---|---|---|
| P1 | per-directory census guard | a declared directory stops being walked (`scripts` → a path that does not exist) | **red** — *"the census walked scriptz and found nothing"*, with per-directory counts |
| P2 | re-derived census floor | the walked population is halved (`spec` dropped from the walk) | **red** — *"inspected 47 files, below the derived floor of 85"* |
| P3 | consumed-artifact digest check | `artifact_digest_mismatches` hollowed to return clean | **red** — *"a changed consumed-artifact digest was not detected; the check matches nothing"* |
| P4 | empty-population guard | the `digested == 0` branch deleted | **red** — *"a consumed-artifact list with no digest in it re-hashed nothing and reported clean"* |
| P5 | twelve-outcome census | the chain stops demonstrating one outcome (`suspect`'s declared state nulled) | **red** — *"these verification outcomes were never demonstrated: [\"suspect\"]"* |

P5 is the one the coordinator's warning is about, and it is worth being precise:
**this repository had no census gap to fill.** All twelve outcomes came from the
chain alone both before and after the deletion — measured pre-deletion, and
independently re-derived from `assurance_chain.py` by the adversarial reviewer,
who matched the table. No replacement demonstrator was written, so there was
nothing to write a tautology into. P5 probes the assertion that was *tightened*
(containment → equality), not a substitute for a deleted source.

A sixth probe, the one-byte edit to the installed
`engineering_assurance/compatibility.py`, is recorded above and is the end-to-end
form: real bytes, real code path, `make pins` exit 1.

## Attacks that bounced

- Deleting `PROOF-legacy-compatibility` does not orphan a `tool_identity`:
  `observe_tool_versions` iterates the declaration's own proof obligations, so
  the identity leaves with the obligation. No hardcoded reference remained.
- The `ci:` prerequisite set is unchanged at 13 and the test that reads the
  expanded list still passes. `compat-view` was never a `ci:` prerequisite; it
  hung off `assurance`.
- `spec/evidence/suites.md` survives intact at 7 of 7. No suite was defined over
  the compatibility view; `SUITE-007` is `make assurance`, which still runs.
- `NFR-002` and `AP-001` were swept for retained-evidence arguments and carry
  none. `NFR-002`'s *"retained corpus"* is `corpus/west-v1`, a different subject
  that is untouched.
- The mirror-registry scan and both its controls are unaffected: it reads
  `consumed_artifacts[].path`, which is still a populated list.
