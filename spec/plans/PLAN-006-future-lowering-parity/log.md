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
