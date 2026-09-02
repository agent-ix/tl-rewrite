---
id: SR-007
title: Closing code review of the tl-rewrite shared assurance migration
type: SpecReview
analysis: code-review
scope: assurance/**/*, scripts/**/*, examples/**/*.rs, tests/**/*.rs, corpus/**/*, schemas/**/*, Makefile, .github/workflows/ci.yml, Cargo.toml, src/lib.rs
review_set: all
---

# Closing code review of the tl-rewrite shared assurance migration

## Summary

An independent agent was commissioned with one instruction — **find false
greens** — and reviewed the branch by executing probes rather than reading. It
returned **3 high, 5 medium and 9 low**, and 14 attacks bounced.

Two of the three highs were claims this change made about itself, and both were
the exact shapes this program has been finding all campaign: a mapping that
turned "I could not read this" into `passed`, and an isolation probe that could
not fail. The self-review that preceded it found and fixed three of its own
probe defects and still missed these.

Every finding is dispositioned below. Twelve are FIXED, four ACCEPTED with
rationale, and one DEFERRED to a linked issue.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-701 | high | FIXED. `ROW_RESULTS` was one global table mapping `malformed` and `unsupported` to `passed`, argued from the two producers that had earned it. A third reached it: appending a non-checksum line to `corpus/west-v1/SHA256SUMS` makes `check_provenance.py` exit 1 with `matched: false` and an outcome set of `['malformed','pass']`, and the chain sealed `PROOF-provenance-integrity: passed` and exited 0. The table is now keyed by proof obligation; `malformed` fails by default and only the producer whose obligation is to name it maps otherwise. Re-probed: chain exit 1. | FR-006-AC-2, NFR-003-AC-1 |
| FND-702 | high | FIXED. `tests/shared_assurance.rs`'s producer-isolation test passed with `producer_shims(&producers, &[])` — no shims at all — because an empty invocation log and an absent shim are the same observation. The shims now write version queries to a second log and the test requires it non-empty and to name `cargo` and `rustc`. Re-probed: exit 101, *"no shim answered a version query"*. | FR-006-AC-2, NFR-003-AC-2 |
| FND-703 | high | FIXED. The pinned coverage figures cannot see an unbacked matrix row: repointing `FR-001` at `TC-901..TC-904` leaves `totals` at 72/72 while `unbacked_rows` gains an entry. `derive_result` and TC-025 now read that field. Re-probed with the export regenerated: chain exit 1, TC-025 exit 101. `status_lies` is kept and its known-dead scope for the Functional and Stakeholder tables is stated in both places rather than left for a reader to discover. | FR-006-AC-3 |
| FND-704 | high | FIXED. Raised by the campaign coordinator from two sibling repositories that had each already "fixed" this class. `make assurance-inputs` invokes five programs — `cargo`, `rustup`, `quire`, `python3` and `.venv-assurance/bin/python` — and PATH shims can cover only three: `python3` is the driver's own interpreter and `quire` is invoked by Quoin itself inside `quoin evidence record`. A driver that runs a producer and discards the output also leaves no trace on disk, so the input-digest check cannot see it either. The driver now funnels every subprocess through `_execute`, records it, and refuses anything outside `quoin` and five declared version observations. Re-probed with `quire coverage`, `cargo build`, `python3 scripts/check_provenance.py`, `.venv-assurance/bin/python scripts/legacy_evidence_view.py` and a driver that regenerates a deleted input: all five exit 101. | FR-006-AC-2, NFR-003-AC-2 |
| FND-705 | medium | FIXED. The `non-conclusive-reasons-stay-distinct` scenario compared `[] == []` and `0 == 0`; removing all five bounded-domain cases left it green and still reported `inconclusive` demonstrated. It now carries the same `> 0` guard its four sibling scenarios have, and TC-028 pins all four corpus counts. | FR-006-AC-5 |
| FND-706 | medium | ACCEPTED, with the overclaim removed. Five of seven sealed tool versions are this crate's `Cargo.toml` version rather than an observed one — the three producers under `examples/` and the two scripts under `scripts/` have no `--version` to ask. `observe_tool_versions`' docstring said "observed" of all seven. It now distinguishes the two sources and the chain report carries `tool_version_sources` per identity. The values are correct; the word was not. | NFR-003 |
| FND-707 | medium | FIXED. `legacy_evidence_view.py --mutation-probes` reported `0/0`, `matched: true` and exit 0 over an empty probe dict, because `detected == len(results)` is true of an empty list. The probe set is now declared as `REQUIRED_PROBES` and a set mismatch is exit 2. Re-probed: exit 2. | FR-006-AC-4 |
| FND-708 | medium | FIXED. `accepted([])` is `True`, and `artifact_digest_mismatches` returned clean over a `consumed_artifacts` list with no digests. Both now refuse an empty subject. Re-probed: exit 1 in both cases. | FR-006-AC-1 |
| FND-709 | medium | FIXED. The counterexample corpus cross-checked its `counts` block against its own `cases` list, which a single coordinated edit moves on both sides, and only `mismatch == 5` was pinned outside the file. TC-028 now pins `non_conclusive`, `malformed` and `total`, and TC-024 pins the three rule counts. | FR-006-AC-6 |
| FND-710 | medium | FIXED. Raised by the campaign coordinator. The control `intake-accepts-unchanged-bytes` read the same boolean as its paired scenario `retain-producer-output`. It now asserts a different fact — that Quoin's intake returned 0 and named a retention directory — which is what makes the paired refusal meaningful. | FR-006-AC-5 |
| FND-711 | low | FIXED. `CLAUDE.md` said `.IGNORE:` neuters "ten of the fourteen" `ci` prerequisites, contradicting the four canonical records. Corrected to the measured 13 of 13. | NFR-003 |
| FND-712 | low | FIXED. The `.IGNORE:` measurement recorded "27 ignored recipe failures" and named no command, so it was not reproducible; the reviewer measured 28 and 22 with different variable sets. All four records now carry the exact command and the number it yields, **28**, together with which prerequisite fails how. | NFR-003 |
| FND-713 | low | FIXED. `check_shared_pins.py`'s mirror scan silently skipped a file it could not decode. An unscanned file is now reported as an offender. | FR-006-AC-1 |
| FND-714 | low | FIXED. Four adapter probes caught a bare `ChainError`, so a refusal for the wrong reason counted as a detection. Each now asserts the message names its cause. | FR-006-AC-5 |
| FND-715 | low | FIXED. `matched = all(...)` over scenarios, controls and probes is `True` when all three are empty; a chain that demonstrated nothing would have reported a match. Non-emptiness is now required in the driver as well as in the test. | FR-006-AC-5 |
| FND-716 | low | FIXED. The source census walked nine directories and three named root files, so a reintroduced validator at the repository root would not have been inspected, and `schemas/README.md` claimed a completeness the census did not have. It now walks ten directories plus every root file, discovered rather than listed, and the README says what it actually does. | FR-006-AC-7 |
| FND-717 | low | FIXED. `schemas/README.md` said `make ci` runs no JSON-Schema validation. `jsonschema` is present in `.venv-assurance` because `engineering-assurance` depends on it, and `map_pgm01_bytes` validates its own view against the release's packaged schema during `make compat-view`. The narrower and true claim — no script in this repository imports it — replaces the false one. | FR-006-AC-7 |
| FND-718 | low | FIXED. Raised by the campaign coordinator. A retained record removed from `evidence/` produced an uncaught `FileNotFoundError` traceback. A gate that crashes has not diagnosed anything; it is now a named `ViewError` saying the archive changed. | FR-006-AC-4 |
| FND-719 | low | ACCEPTED. `witnessReplayed` and the two replayed verdicts are producer-written, and the replay is independent of the enumeration but not of the evaluator — both sides call `tl_mltl::evaluate_closed`. A second evaluator would be a second implementation of the thing under test. The producer hard-fails on any disagreement, and four chain-level attacks on the field are all detected. | FR-006-AC-6 |
| FND-720 | low | ACCEPTED. `requirements-assurance.txt` is newer than `.venv-assurance/bin/python`, so `make assurance-env` rebuilds the environment from a git tag inside `make ci`. That is the rule doing its job — the environment must be rebuilt when the pin changes — and on any fresh clone the environment does not exist at all. Recorded rather than suppressed by touching mtimes. | FR-006-AC-1 |
| FND-721 | low | ACCEPTED. Nothing detects a rewritten `scripts/assurance_chain.py`; a driver cannot police its own replacement. Git history and pull-request review are the boundary, which is what `CONTRIBUTING.md` has always said. | NFR-003 |
| FND-722 | medium | DEFERRED to `agent-ix/quire-contract-ir#21`. `quire coverage --strict` reports `status-column-matches-nothing` and exits 0, so a mis-statused row is invisible for the Functional and Stakeholder tables. Upstream and not repo-controllable; the 72/72 figure is a backing figure, not a status-verified one, and both the driver and TC-025 say so. | TM-001 |

## What held

Fourteen attacks bounced, and they are worth naming because they are the ones a
reader would otherwise have to take on trust: total producer failure, absent,
empty and unreadable inputs, all four counterexample-witness attacks, the
bounded-equivalence limitation, the `ci` prerequisite set, the rule corpus's
binding to the catalog, absent tools classifying as unknown, all three
`recoverable_at` claims in `assurance/pins.json`, both `mirror_references`
branches, both retained-schema digests, and the workflow pins.

## Repin

Both temporal dependencies moved onto revisions reachable from their own `main`
in this change: `tl-syntax` `740182f1` to `953ee825`, `tl-mltl` `fe1c620d` to
`f7eb8bdf`. The order was forced and measured — an earlier attempt to move only
`tl-syntax` produced E0308 at four call sites in `src/equivalence.rs`, because
this crate passes `tl_syntax::Formula` values into `tl_mltl::evaluate_closed`
and both crates must resolve the same revision. `agent-ix/tl-rewrite#10` is
closed by this change.
