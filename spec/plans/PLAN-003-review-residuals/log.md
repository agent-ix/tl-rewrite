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
- **Authorial cross-check (superseded).** The first residual pass added a
  top-level `census_controls` block to the change declaration. Task-003 removes
  it: Quoin's sealed record schema has no control-metadata field, so retaining
  it would be an unsealed local duplicate. FR-006-AC-7 and TC-029 now own the
  executable controls, while Quoin seals the requirement statement and sources.
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
- **2026-09-04 — Round-two review remediation verified.** Replaced the fragile
  lower bound with exact population equality, clarified the historical census
  record and Make-control scope, and corrected the isolation-record wording.
  The focused census and complete local
  `make ci CARGO_TARGET_DIR=target/cargo-review` gate passed; hosted CI was not
  dispatched. The plan remains in progress until the changed head is
  independently re-reviewed and merged.
- **2026-09-08 — Issue #22 task decomposition.** The post-merge independent
  review identified bounded follow-up work outside the original issue #19
  scope. Three typed tasks now allocate that work without treating authorial
  verification records as independent clearance. Implementation and a new
  independent review remain outstanding.
- **2026-09-08 — Artifact-truth task completed.** SR-015, SR-013, and SR-014
  now state their authorial status, identify `d4670c25` as the historical
  candidate for their execution observations, and grant neither independent
  clearance nor merge authority. This record update does not substitute for the
  independent review still required for the current branch.
- **2026-09-08 — Census-authority task completed.** The shared Quoin schema was
  inspected: its strict record permits requirements, proof obligations,
  preservation constraints, and unknowns, but no free-form control metadata.
  The unsealed `census_controls` duplicate was therefore removed rather than
  imitated as a local record field. FR-006-AC-7 and TC-029 now own the controls;
  the focused raw-byte census and local specification gate passed at the
  resulting candidate. Hosted CI was not dispatched.
- **2026-09-08 — Shared-input isolation task completed.** A private guard token
  now serializes all shared-input readers and mutators, and token-taking helpers
  make omission from established stateful paths fail at compile time. Scratch
  and shim cleanup reports failure rather than silently continuing. No runner,
  collector, envelope, or retention layer was introduced.
- **2026-09-09 — Post-#30 review remediation.** Rebased onto the merged review-id
  census, retained the reallocated residual record at SR-015, and moved this
  branch's nine new full-corpus reviews to SR-017 through SR-025. The deleted-name
  raw-byte probe now has a literal expected side independent from its scanner
  array, the deny predicate and reviewed path identities are independent, and a
  poisoned serialization lock no longer masks the actual state of shared inputs.
  A fresh exact-head review and merge remain outstanding; hosted CI was not
  dispatched.
