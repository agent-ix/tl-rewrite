---
id: SR-005
title: Code review of the tl-rewrite shared assurance migration
type: SpecReview
analysis: code-review
scope: assurance/**/*, scripts/**/*, examples/**/*.rs, tests/**/*.rs, corpus/**/*, schemas/**/*, Makefile, .github/workflows/ci.yml, Cargo.toml, src/lib.rs
review_set: all
---

# Code review of the tl-rewrite shared assurance migration

## Summary

This review covers the migration under issue #9 from a repository-local evidence
framework onto Engineering Assurance 0.2.0, quire-cli 0.31.0 (engine 0.46.0),
quoin 0.23.1 and ix-flow 0.0.4. It was performed against the branch head with
`make ci` green and every gate re-run, not read-only.

The rewrite engine is unchanged. What changed is how its results are declared,
transcribed, retained and verified: four producers write structured results, one
driver reads those bytes and seals them through the pinned Quoin CLI, and the
eleven retained evidence records are read through the pinned Engineering
Assurance mapping instead of a local verifier.

Three defects were found and fixed during the review, all of them in this
change's own checks rather than in the engine. Two were probes that could not
fail; one was a wire constant that named a dependency revision the build had not
used.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-501 | high | Resolved: `TL_MLTL_REVISION` declared `da2c7704`, a revision reachable from no branch or tag, while `Cargo.toml` had moved to `fe1c620d`. Every `ConformanceReport` therefore named an evaluator revision that had not produced its verdicts, and the only test touching the field compared the constant to itself. `scripts/check_provenance.py` now requires `src/lib.rs`, `Cargo.toml` and `Cargo.lock` to agree for both temporal dependencies, and TC-030 restores the stale value in a scratch copy and requires the census to go red. | NFR-002-AC-2, NFR-003-AC-5 |
| FND-502 | high | Resolved: `producer_shims` in `tests/shared_assurance.rs` did not clear its directory, so shims written by an earlier run stayed on disk. A mutation that stopped writing them entirely still found the old ones on `PATH` and the isolation test passed. This is the same shape as the sibling finding the test was copied to avoid. The helper now removes the directory first, and the "shims absent" probe turns it red. | FR-006-AC-2, NFR-003-AC-2 |
| FND-503 | medium | Resolved: the isolation test asserted only that no producer was invoked, not that the driver left its inputs alone. Shimming `quire` does not close that — `quoin evidence record` itself shells out to `quire coverage`, which is a static export and not a producer execution. Every file under `target/assurance` is now digested before and after the chain runs, and a probe that makes `require_inputs` fabricate a missing input rather than raise turns the test red. | FR-006-AC-2, NFR-003-AC-1 |
| FND-504 | medium | Resolved: deleting a prerequisite from `ci:` removed a whole enforcement layer while every remaining test stayed green. TC-029 now reads the expanded `ci` prerequisite list and requires it to be exactly the declared thirteen. Deleting `test` and deleting `assurance` both turn it red. | FR-006-AC-7 |
| FND-505 | medium | Resolved: `assurance/pins.json` first described both retained evidence schemas as "frozen because retained envelopes name them". That is false for one of them. Measured: all 11 envelopes name the manifest schema by its exact committed digest, while the input schema's committed bytes match none of the three digests the envelopes reference — it was tightened twice after those records were sealed. Both statuses are now stated separately, the three referenced digests are mapped to the revisions at which they are recoverable, and both digests are re-derived by `check_shared_pins.py` and pinned by TC-029. | NFR-002-AC-2, FR-006-AC-7 |
| FND-506 | medium | Resolved: the counterexample lane could have degraded into a boolean. `examples/counterexample_evidence.rs` now replays every recorded trace against both documents through `tl_mltl::evaluate_closed`, outside the enumeration that produced it, and refuses a witness whose two verdicts agree or whose trace is empty. The chain asserts the witness count against the corpus declaration rather than against the producer, and requires the witnesses to survive byte-identically into what Quoin retained. | FR-006-AC-6, FR-004-AC-1 |
| FND-507 | medium | Accepted: `derive()` in `scripts/legacy_evidence_view.py` now refuses a replacement that writes back the value already present. A sibling's first tamper probe did exactly that, changed no byte, and counted as a detection. | FR-006-AC-4 |
| FND-508 | low | Accepted: `docs/DER-001-rule-derivations.md` gained typed frontmatter so `quire validate --strict` covers it. It was previously outside the validated set. | NFR-002 |
| FND-509 | low | Deferred: the `tl-syntax` repin to `953ee825` cannot be performed while `tl-mltl` at merged `main` still pins `740182f1`; the two crates must resolve the same revision because `src/equivalence.rs` passes `tl_syntax::Formula` values into `tl_mltl::evaluate_closed`. Measured as E0308 at four call sites. Filed as agent-ix/tl-rewrite#10. | AA-001 |
| FND-510 | low | Deferred: the Make execution-control guard is gone and `.IGNORE:` neuters all 13 `ci` prerequisites. Recorded in four places and filed as agent-ix/tl-rewrite#11 rather than closed. | NFR-003 |

## Boundary review

`scripts/assurance_chain.py` runs exactly one command, `quoin`, guarded by a
`shutil.which` pre-flight. `scripts/legacy_evidence_view.py` runs exactly one,
`git status --porcelain -- evidence`, and imports the mapping rather than
implementing one. `scripts/check_shared_pins.py` observes versions and delegates
every verdict to `engineering_assurance.compatibility`. None of the three runs a
producer, and the isolation test asserts that behaviourally rather than by
grepping the source.

## Removed

3,665 lines across 27 whole-file deletions, in a commit of its own after both
paths were exercised at the same candidate revision. Every removed path is
asserted absent by name in TC-029.

## Not reviewed

Rewrite, replay and equivalence behaviour, which this change does not touch;
`git diff 74aa3f1 HEAD -- src/` is a two-line constant correction.
