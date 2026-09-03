---
type: log
title: "PLAN-003 update log"
description: "Implementation and verification record for issue #19."
---
# PLAN-003 update log

## History

- **2026-09-03 — Plan opened.** Re-read issue #19 and the exact-head reviews of
  PRs #17 and #18 before changing the merged tree.
- **Census design.** Extracted the existing Git enumeration into one helper used
  by both the repository census and a real fixture repository. The fixture uses
  an empty template override, stages and then neutralizes a fixture-local global
  excludes file, and carries a phony-only preferred `GNUmakefile` control.
- **Authorial cross-check.** Added top-level `census_controls` to the change
  declaration and labelled it unsealed. The executable sets remain in the test;
  the two representations are separately reviewable, not independent authority.
- **Probe isolation.** Removed the dead post-creation ownership assertion and
  made store containment resolve an existing real store leaf, falling back only
  when that leaf is absent.
- **Boundary retained.** No hosted CI was dispatched and no repository-local
  Make execution-control framework was added.
- **Verification.** The focused census passed, the complete shared-assurance
  binary passed 10/10, and local `make ci CARGO_TARGET_DIR=target/cargo-review`
  passed with 38 Rust tests and 68/68 Quire rows backed. The first full run found
  malformed new SpecReview structure; the required summaries and canonical
  finding columns were corrected before the passing run. Mutations narrowing
  `compat-view` back to `compat-view:`, removing `git init --template=`, and
  widening historical prose to `docs/` each failed TC-029 at its intended
  assertion. The passing isolation probe exercised an existing real
  `target/assurance-store` leaf. Hosted CI was not dispatched.
