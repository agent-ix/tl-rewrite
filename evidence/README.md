# Retained evidence

The eleven records in this directory are immutable. They were produced by a
local evidence collector that this repository no longer has: `collect_evidence.sh`,
`build_evidence_envelope.py`, `finalize_collection.py`, `verify_evidence.sh`,
`verify_evidence_manifest.py`, `verify_evidence_history.py`, `evidence_profile.py`
and `check_assurance_anchor.py` were removed by the migration under issue #9,
along with `ANCHORS` and `RETRACTIONS.json`'s enforcing readers. **The verifier is
what was removed; the record is not.** Every byte here, including `ANCHORS` and
`RETRACTIONS.json` themselves, is unchanged.

## How to read them now

`scripts/legacy_evidence_view.py` hands each envelope to
`engineering_assurance.verification_semantics.map_pgm01_bytes` from the pinned
release and reports what came back. It implements no mapping of its own.

The answer, measured rather than inherited: all eleven envelopes declare
`quire.derivation-evidence/v1`, and the pinned mapping covers
`quire.pgm01-evidence` v1 and v2 only, so every one of them reads as
`incompatible` with the reason *"unknown PGM-01 schema version"*. That is the
mapping declining to interpret a shape it has never seen. It is reported as it
stands — not converted into a pass, not converted into a defect of these
records, and not a licence to write a local mapper. Filed upstream as
`agent-ix/engineering-assurance#21`. The sibling temporal repositories reported
the identical refusal over their own envelopes; `quire-analyze`, whose retained
artifacts are Markdown, got `unreadable` instead, which is why the answer is
measured per repository.

## Integrity boundary

Git history and pull-request review. That is what `CONTRIBUTING.md` has always
said, and the compatibility view now asks Git directly — `git status --porcelain
-- evidence` — rather than maintaining a second local manifest and implying a
stronger claim than it can make. A tree where Git cannot be run reports that
fact rather than an empty list, because "nothing changed" and "nobody looked"
must not be the same answer.

## What these records do not establish

They inform a pending human source-release decision. They neither prove
arbitrary rewrite schemas nor approve, validate, qualify, accredit, certify, or
publish a consuming project. Bounded agreement over a horizon-complete domain is
bounded evidence and nothing more.
