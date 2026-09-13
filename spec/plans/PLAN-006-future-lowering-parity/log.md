---
type: log
title: "PLAN-006 update log"
description: "Implementation and verification record for issue #35."
---
# PLAN-006 update log

## History

- **2026-09-12 — Plan opened.** tl-syntax#40 merged as `8dc18ee` and
  tl-parse#31 as `9ca856b`. tl-mltl `4bff387` still pins tl-syntax `26b801d`,
  so the lowering and v2 parser enter as renamed dev-only dependencies and
  lowered graphs cross into the production type through the formula v1 wire.
- **2026-09-12 — Task-001 completed.** `tests/future_lowering_parity.rs` adds
  TC-041 through TC-045. A hand mutation of the direct builder turned four of
  the five tests red. The first endpoint mutant widened the Until/Release node
  and survived: a witness at `b+1` is already implied by (W) or excluded by (M)
  the unary disjunct, so that mutant is a semantic no-op. The mutant now widens
  the Globally/Future node, which reaches a bounded counterexample.
- **2026-09-12 — Gate and PR.** `make ci` exited 0 at `049fd10`; PR #36 opened
  with `Refs #35`. The released Quire rejects `Coverage Status` headers, so the
  matrix uses one `Status` column. The coverage pin moved to 99 (main measures
  89 plus the 10 FR-008 rows) and the removal census to 165 (+7 files).
- **2026-09-12 — Review findings fixed.** PR-time `rust-review` and
  `gap-analysis` ran. Fixes: parity assertions now lead with graph identity; the
  scanner covers `examples/`, quoted and aliased node kinds, the catalog, Quire
  and FRETish tokens, and every production dependency section, each with a
  synthetic control; TC-041 adds a `u32::MAX` interval pair; TC-043 adds a
  nested source and an associativity control; TC-045 adds operand-swap,
  narrowed-start, and dropped-unary mutants under an exhaustive class match;
  `scripts/check_provenance.py` now ties the production tl-syntax pin to the
  revision tl-mltl locks and refuses undeclared locked revisions, with TC-030
  probes; FR-008 criteria, the spec scope, and the pin comments were corrected.
  Run against the previous provenance check, the new TC-030 probe failed: that
  check accepted a production pin moved to the dev-only revision.
