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

The change deletes 594 tracked files: the retained evidence archive, its only
reader, the fixtures and schemas that served it, and the chain proof obligation
that read it. It is authorised by `agent-ix/engineering-assurance#7`, section
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
| Total | **594 files, 15,108 deleted lines** |

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

## Mutation probes: 44 → 33, and why it fell

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

One is added: the `digested == 0` branch of `artifact_digest_mismatches` is now
probed directly, because that branch is what would have caught the deletion
emptying the list, and an unprobed guard is a guard nobody has seen fire.

**44 → 32 by deletion, plus 1 added = 33.** The count fell because there is less
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
| FND-903 | high | **FIXED.** `spec/assurance/AA-001.md`'s **Sufficiency Decision** gated the top-level release claim on *"retained evidence names the exact candidate"*, eleven lines above a section that now disclaims any retained evidence. Left as written the release gate was unsatisfiable. Restated against the sealed change-assurance record, which is bound to `--candidate-revision` on every `make assurance` run. Raised by the campaign coordinator from `tl-syntax`, which shipped the same shape. | FR-005-AC-3 |
| FND-904 | medium | **FIXED.** `spec/assurance/MP-001.md`'s collection procedure listed *"canonical PGM-01 validation against one immutable revision"*. That validation was performed against the deleted archive. Removed rather than restated over an empty population, and the removal is named in the document. | FR-006-AC-7 |
| FND-905 | medium | **FIXED.** `tests/wire_records.rs::human_authority_and_qualification_boundaries_remain_open` (TC-019) inspected `evidence/README.md`, which the deletion removed, and that file was the **only** source of the required token `pending` — it said the retained records *"inform a pending human source-release decision"*. Caught by `make ci`, not by reading. The premise was checked before weakening: the property is still true, so rather than dropping the token the statement moved to `AA-001`'s Human Decision section, which is the document that owns the claim. Six properties over four documents, not five over four. | FR-005-AC-3, StR-001-VC-1 |
| FND-906 | medium | **FIXED.** `assurance/pins.json`'s `consumed_artifacts` would have been emptied of every digest-pinned entry. See the section above: replaced with a live pin on the module the surviving caller imports, and probed red with a one-byte upstream edit. Approach taken from `quire-analyze`, which solved the same problem the same way. | FR-006-AC-1 |
| FND-907 | medium | **FIXED.** `"disposition": "moot"` was written into `assurance/change-assurance.json` for the derivation-evidence unknown. Quoin's vocabulary is `open | accepted | deferred | resolved`, and it refused to seal the record — *"unknown disposition"*. Caught by running the chain. Set to `accepted`, which is the honest word: the upstream mapping gap is **not** resolved and nothing here fixed it; what changed is that this repository no longer has a subject it applies to. The entry is kept in the record rather than deleted with the envelopes so a reader can see the gap was accepted and never closed. | FR-006-AC-1 |
| FND-908 | low | **FIXED.** The new docstring in `scripts/check_shared_pins.py` and the new prose in `assurance/README.md` and `CLAUDE.md` named the deleted reader and obligation by their exact identifiers, which the TC-029 source census correctly refused. Caught by the census, which is the census working. Reworded to describe the caller and the obligation rather than name them; the identifiers survive only where they are the subject of an assertion or a record. | FR-006-AC-7 |
| FND-909 | low | **ACCEPTED.** `scripts/check_provenance.py`'s module docstring still names `da2c7704`. It is not a live requirement on that revision — it is the narrative of the defect the check exists to catch, and `TC-030` restores that exact value in a scratch copy to prove the check goes red. Removing it would remove the reason the check exists. | NFR-003-AC-5 |
| FND-910 | low | **ACCEPTED.** `scripts/assurance_chain.py`'s docstring still says *"Nothing is written under `evidence/`"*. The claim is about the driver's own behaviour — it is not a retention store — and remains true and meaningful now that no such directory exists. | FR-006-AC-2 |

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
