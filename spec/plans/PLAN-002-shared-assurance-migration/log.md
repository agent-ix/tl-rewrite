---
type: log
title: "PLAN-002 update log"
description: "What changed in the tl-rewrite shared-assurance migration, and when."
---
# PLAN-002 update log

## History

- **2026-09-02** - Bundle opened for issue #9 on top of the v0.1 rewrite branch,
  because the engine, catalog and corpora this migration preserves exist only
  there and `main` carries a stub crate root.
- **Inventory and pins.** Produced the keep/replace/delete/defer inventory.
  Declared engineering-assurance v0.2.0 in `requirements-assurance.txt` and
  `assurance/pins.json`. Repinned the hosted workflow's `@agent-ix/quoin` from
  0.22.5 — a version the packaged compatibility matrix names explicitly
  incompatible — to 0.23.1, and added the missing `ix-flow@0.0.4`.
- **Upstream repin, blocked then unblocked, both measured.** The `tl-syntax`
  repin from `740182f1` to the merged `953ee825` was attempted first and failed:
  cargo reported E0308 at four call sites in `src/equivalence.rs`, because
  `tl-mltl` at its merged `main` still pinned `740182f1` and this crate hands
  `tl_syntax::Formula` values straight to `tl_mltl::evaluate_closed`, so both
  must resolve the same revision. It was reverted and recorded as an open
  unknown. `tl-mltl`'s own migration then merged as `f7eb8bdf`, carrying its
  repin to `953ee825`, and both pins moved together in one change. The retained
  records' `da2c7704` remains fetchable only from
  `agent-ix/tl-mltl` `origin/retain/tl-mltl-v0.1-da2c7704`, which must not be
  deleted (`agent-ix/tl-mltl#11`).
- **Corrected a wire constant.** `TL_MLTL_REVISION` named `da2c7704`, a revision
  reachable from no branch or tag, while `Cargo.toml` had moved to `fe1c620d`.
  Every `ConformanceReport` therefore attributed its verdicts to an evaluator
  that had not produced them, and the only test touching the field compared the
  constant to itself. Corrected, and `scripts/check_provenance.py` now requires
  `src/lib.rs`, `Cargo.toml` and `Cargo.lock` to agree.
- **Producers.** Added `examples/rule_conformance.rs`,
  `examples/counterexample_evidence.rs` and `examples/normalization_sweep.rs`,
  plus `corpus/rules/manifest.json` and `corpus/counterexamples/manifest.json`.
  The rule corpus is bound by SHA-256 to the fixtures `tests/rule_evidence.rs`
  constructs, in both directions.
- **Shared intake.** Added `assurance/`, `scripts/assurance_chain.py`,
  `scripts/check_shared_pins.py`, `scripts/legacy_evidence_view.py` and
  `tests/shared_assurance.rs`. Rewrote the `Makefile` as native orchestration.
- **Make execution controls.** The parse-time guard and
  `scripts/check_failure_propagation.py` were removed with the collector they
  protected. The cost was measured on this repository's own Makefile rather than
  inherited, and with the command named so the number can be re-derived: under a
  single `.IGNORE:` line `make ci` goes from exit 2 to exit 0 after 28 ignored
  recipe failures, with all 13 `ci` prerequisites reporting success. Recorded in
  NFR-003, AA-001, the change-assurance declaration and the Makefile header, and
  filed as `agent-ix/tl-rewrite#11`.
- **Adversarial review.** An independent agent, instructed only to find false
  greens, returned 3 high, 5 medium and 9 low. Two of the highs were real false
  greens this change had introduced: a global outcome table that mapped
  `malformed` to `passed` for producers that had never argued for it, so a
  `check_provenance.py` run that exited 1 saying it could not read the WEST
  checksum manifest was sealed `passed`; and a producer-isolation test that still
  passed with the shims removed entirely. The third showed that the pinned
  coverage totals cannot see an unbacked matrix row. All dispositioned in SR-007
  and SR-008.
- **Dual run and deletion.** Both paths exercised against the same candidate
  revision, the result recorded as observed, and the local evidence framework
  deleted in a separate final commit.
