---
id: SR-097
title: "tl-rewrite#57 drop dangling PGM-01 citations review"
type: SpecReview
analysis: base
scope: "agent-ix/tl-rewrite@2e006ba8021783060b0e1798a3b7fdb30e88e4b8; diff against origin/main e48cf0b; CLAUDE.md; README.md; assurance/pins.json; spec/assurance/AA-001.md; spec/assurance/MP-001.md; spec/evidence/suites.md; spec/requirements/FR-001-rule-catalog.md; spec/requirements/NFR-002-provenance-boundary.md; spec/reviews/SR-004-pgm01-reconciliation.md (deleted); spec/spec.md; src/lib.rs"
review_set: subset
---

# tl-rewrite#57 drop dangling PGM-01 citations review

## Summary

Ticket: TL-272. PR: agent-ix/tl-rewrite#57, branch `chore/drop-pgm01-citations`,
reviewed at `2e006ba`. Methods run in this one document, scoped to the diff
against `origin/main` (`e48cf0b`, which is also the merge base):
- spec-review, the base checklist over the changed spec text;
- gap-analysis, planless;
- rust-review, over the removal of `pub const PGM01_POLICY_REVISION` from
  `src/lib.rs`.

PGM-01 was a governance and evidence-policy standard in
`agent-ix/quire-contract-ir`. It was deleted there as tracking ceremony, and the
owner's rule is that tracking ceremony is deleted.

## Verdict

**PASS, clean.**

- **No dangling live references.** No `quire-contract-ir/PGM-01` edge is left.
  The PR removed the ones on MRS-001 and NFR-002; NFR-002's now-empty
  `relationships:` key is gone too. The PGM-01 reference link was removed from
  `spec/spec.md`. The remaining `PGM-01` and `pgm01` text is of two kinds:
  - archival records in `spec/plans/PLAN-001-v0.1/**` and `spec/reviews/**`;
  - literal tool output and identifiers in `assurance/pins.json:24,39,40,51`
    and `assurance/change-assurance.json:420`, such as `map_pgm01_bytes`, the
    refusal reason "unknown PGM-01 schema version", and the file name
    `pgm01-compatibility-view-v1.schema.json`.
- **The SR-004 deletion was ceremony only.** The file held:
  - a PGM-01-R01..R10 mapping;
  - one medium "finding" that restated the open human-review gate, which
    AP-001 and AA-001 already own;
  - no FR, AC or test.

  No file in the tree names SR-004 or `pgm01-reconciliation`.
- **The prose stays accurate.** CLAUDE.md, README.md and AA-001 no longer say
  Engineering Assurance supplies a "PGM-01 mapping". That is true: the only
  reader, `scripts/legacy_evidence_view.py`, was deleted under tl-rewrite#13.
  The MP-001 paragraph that was removed only recorded that PGM-01 validation had
  stopped. `spec/spec.md:88` still says why FR-005's evidence allocation was
  removed.
- **No AC lost meaning.** The edits are in Dependencies sections, frontmatter
  and narrative. No AC row, TC, or test matrix row changed.
- **Rust review of the `pub const PGM01_POLICY_REVISION` removal.**
  - Nothing reads the constant. `grep` over the origin/main tree, excluding
    `spec/reviews/`, finds only its definition at `src/lib.rs:59`. A search of
    every `.rs` and `.py` file under `~/dev` finds only definition sites.
  - `scripts/check_provenance.py` reads `src/lib.rs`, but it checks only
    `TL_SYNTAX_REVISION`, `TL_MLTL_REVISION` and `WEST_REVISION`. It passes at
    the PR head.
  - The crate is `publish = false` at 0.3.0, and CHANGELOG entries are written
    at release time, so no CHANGELOG entry is owed in this PR.
  - `cargo check --all-targets` passes with 0 warnings. It ran in a dedicated
    target dir, which was deleted afterwards.
- **Validation matches origin/main.** The Makefile's `quire validate` output,
  `'spec/**/*.md' 'docs/*.md' --strict --summary`, with paths normalised, is
  identical on origin/main and the PR head. Both exit 0. The only change is the
  document count, 167 to 166, from the deleted SR-004.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope examined

| Unit | Role | Result |
| --- | --- | --- |
| spec/spec.md (MRS-001 relationships, Purpose, FR-005 note, References) | examined | clean |
| spec/requirements/NFR-002-provenance-boundary.md (relationships, Dependencies) | examined | clean |
| spec/requirements/FR-001-rule-catalog.md Dependencies | examined | clean |
| spec/assurance/MP-001.md Collection Procedure | examined | clean |
| spec/assurance/AA-001.md ownership paragraph | examined | clean |
| spec/evidence/suites.md SUITE-007 note | examined | clean |
| CLAUDE.md:44, README.md:87 | examined | clean |
| assurance/pins.json:24 consumed_artifacts_note | examined | clean |
| src/lib.rs PGM01_POLICY_REVISION removal | examined | clean; no readers |
| scripts/check_provenance.py | context_only | reads only the three dependency and corpus constants |
| spec/reviews/SR-004-pgm01-reconciliation.md (deleted) | examined | ceremony only; no inbound links |

## Gap analysis

Plan completion: not assessed.

- No requirement, AC, or TC was added, removed, or reworded.
- Removing the constant takes away public code that had no owning requirement
  and no test. That reduces unowned code; it does not create a gap.

## Remaining pin/digest machinery (out of this PR's scope: blocked on permission)

The PR does not touch this machinery, and it is not a must-fix for this PR:

- `assurance/pins.json` holds the release pins, the consumed-artifact digests,
  and the drift and retained-evidence records.
- `assurance/change-assurance.json`
- `assurance/README.md`
- `scripts/check_shared_pins.py`
- `scripts/assurance_chain.py`
- `scripts/check_provenance.py`
- `requirements-assurance.txt`
- `Makefile` targets `assurance-env`, `assurance-inputs`, `pins`,
  `assurance-chain`, `assurance` and `assurance-record`. `test` depends on
  `assurance-inputs`.
- `.github/workflows/ci.yml`
- `tests/shared_assurance.rs`
- `CLAUDE.md` and `CHANGELOG.md` contain the pin narrative.
- The revision constants `TL_SYNTAX_REVISION`, `TL_MLTL_REVISION` and
  `WEST_REVISION` in `src/lib.rs` are wire fields, and `check_provenance.py`
  cross-checks them against `Cargo.toml` and `Cargo.lock`. They may be
  load-bearing.
- Corpus checksum files: `corpus/west-v1/SHA256SUMS` and
  `fuzz/corpus/infinite_rewrite/SHA256SUMS`. These are read by
  `tests/infinite_owner_corpus.rs` and `tests/infinite_fuzz_seeds.rs`. They are
  content digests over the vendored WEST corpus and this repo's seeds, so they
  may be load-bearing.
