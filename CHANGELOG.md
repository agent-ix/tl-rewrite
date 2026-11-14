# Changelog

All notable user-visible changes to `tl-rewrite` are recorded here. The crate is
distributed as a git source release (`publish = false`); versions are git tags.

## 0.4.0

Part of the second coordinated release of the MLTL crates (`tl-syntax`,
`tl-parse`, `tl-mltl`, `tl-rewrite`). All four crates move to 0.4.0 together.
This crate depends on `tl-syntax` 0.4.0 (release commit
`9607722f24ac1978b5e0e0ceeb0cb2a828e10177`) and `tl-mltl` 0.4.0 (release commit
`011a4324dba88a93eddd21ad9ba25d3c5cdc4bef`), both at exact versions. Changes are
relative to v0.3.0. This is a breaking 0.x release; the API migration inventory
below lists every break reported by `cargo-semver-checks` 0.50.0.

### Documented

- `WEST_REVISION` is kept. It is the commit of the external WEST corpus repository,
  which is not a Cargo dependency, so it duplicates no Cargo pin. Every conformance
  report emits it as `westRevision` and contextual reports fold it into
  `requestSha256`; FR-005 and NFR-002-AC-2 require records to name the WEST identity
  the run used. Removing it would change the wire format. No behavior change.

### Added

- Infinite-trace rewriting behind the `infinite-trace` feature
  (`rewrite_infinite`, `replay_infinite`, `check_infinite_rewrite`, the
  `tl-rewrite.infinite-catalog/v1` and `tl-rewrite.infinite-report/v1`
  records, `infinite_catalog`, `infinite_conformance_disposition`).
- A boundary test for the private `MAX_MATERIALIZED_INSTANTS` ceiling, run
  in-crate; Miri builds use a smaller value so it interprets in reasonable time.
- `make miri` runs the pure-computation suites under Miri on the pinned nightly
  and is part of `make ci` (NFR-006).
- Boundary tests accept each declared limit at its value and refuse one past it
  with the typed error that names the limit.
- Oracle-checked rewrite-equivalence properties, a closed-rewrite fuzz target,
  and the cargo-mutants configuration with a kill-rate gate.

### Fixed

- `rewrite_infinite`, and so `replay_infinite` and `check_infinite_rewrite`,
  refuse a deserialized `FairnessPremisesDocument` whose roots lie outside the
  graph with `InfiniteRewriteFailure::IdentityMismatch` instead of panicking
  on an out-of-range index.

### Removed

No input can produce the removed variants through a validated document, and each was
a fail-closed arm over an invariant the types already guarantee. Removing one is a breaking change to an
exhaustive `match`.

- `ConformanceReason::InvalidInput`. Formula and signal-catalog documents are
  validated at construction.
- The compiled-in dependency revisions: `TL_SYNTAX_REVISION` and
  `TL_MLTL_REVISION`, and the `syntax_revision` field of `RewriteReport`,
  `ConformanceReport` and `PastConformanceReport`, and the `evaluator_revision`
  field of `ConformanceReport` and `PastConformanceReport`. The pinned
  revisions are recorded in `Cargo.toml` only.
- `PGM01_POLICY_REVISION`, a policy revision constant no code consumed.
- **Wire break (serialized records).** The record types reject unknown fields, so
  0.4.0 refuses records written by v0.3.0 that still carry a removed field.
  Schema identities were not bumped, by decision for the pre-release 0.x line.
  Deleting the removed keys from the retained bytes (a byte-level edit that keeps
  the key order; re-serializing through a map reorders keys and fails the exact
  canonical encoding check) makes the record decode, but a record that
  carries a contextual request digest no longer reproduces under 0.4.0, so
  regenerate those. What was checked, by writing records with the v0.3.0 tag and
  decoding them with 0.4.0 (v0.3.0 commit `9e656c39`, one closed-trace
  formula, one contextual catalog and context, one past history):
  - `tl-rewrite.report/v1`: Migration: delete `syntaxRevision` from the retained bytes (keep the rest as it is) or regenerate the report; 0.4.0 refuses v0.3.0 reports that carry `syntaxRevision`. Replay-proven: the v0.3.0 report is refused with `unknown field syntaxRevision`; with that one key deleted at the byte level it decodes, re-serializes to the same bytes, and `replay` under 0.4.0 returns `Verified`, so its digests still hold.
  - `tl-rewrite.conformance/v1`: Migration: delete `syntaxRevision` and `evaluatorRevision` from the retained bytes (keep the rest as it is) or regenerate the report; v0.3.0 reports that carry either are refused. Replay-proven: refused with `unknown field evaluatorRevision`; with both keys deleted it decodes and equals the report 0.4.0 regenerates for the same inputs.
  - `tl-rewrite.report/v2`: Migration: regenerate the report with 0.4.0; a v0.3.0 report that carries `syntaxRevision` is refused, and with the key deleted it decodes but `replay_with_context` under 0.4.0 returns `Mismatch`, so its digests no longer hold. Replay-proven for the refusal, the decode and the mismatch; that the cause is the contextual request digest is reviewer-checked.
  - `tl-rewrite.conformance/v2`: Migration: regenerate the report with 0.4.0; v0.3.0 reports that carry `syntaxRevision` or `evaluatorRevision` are refused, and with both keys deleted the report decodes but its `requestSha256` differs from the one 0.4.0 computes (every other field is equal). Replay-proven.
  - `tl-rewrite.past-conformance/v2`: Migration: delete `syntaxRevision` and `evaluatorRevision` from the retained bytes or regenerate the report; v0.3.0 reports that carry either are refused. Replay-proven for the refusal and for decoding after deleting both keys; digest agreement with a regenerated report was not checked, so that part is reviewer-checked.
  - `ConformanceReason::InvalidInput` (wire value `invalid_input`, in `tl-rewrite.conformance/v1` and `/v2`): Migration: a retained report whose `reason` is `invalid_input` is refused; regenerate it, since a validated document can no longer produce that reason. Reviewer-checked, not replay-proven.
  - `tl-rewrite.replay/v1`: carries digests and a status only, so v0.3.0 replay reports still decode (replay-proven). `tl-rewrite.replay/v2` is the same shape and is reviewer-checked.
- `RewriteStatus::Failed` and `InfiniteRewriteFailure::Internal` (both introduced
  after v0.3.0 and removed before this release, so they are not API breaks against
  v0.3.0). The graph remap and compaction paths index by the topological order the
  owner already enforces, and a rebuilt graph the owner refuses past its node
  ceiling is `InfiniteRewriteFailure::Budget(BudgetKind::Nodes)`.
- `InfiniteConformanceReason::InvalidTrace`, likewise introduced and removed
  after v0.3.0. A lasso trace and an infinite formula each admit only the
  infinite-trace profile and the event-position clock.

### API migration inventory

- `enum_no_repr_variant_discriminant_changed` `ConformanceReason::EvaluatorError`: Migration: do not cast `ConformanceReason` variants to integers with `as`; discriminants shifted when variants were removed, so match on the variant or use an explicit mapping.
- `enum_no_repr_variant_discriminant_changed` `ConformanceReason::HorizonLimit`: Migration: do not cast `ConformanceReason` variants to integers with `as`; discriminants shifted when variants were removed, so match on the variant or use an explicit mapping.
- `enum_no_repr_variant_discriminant_changed` `ConformanceReason::OriginalBinding`: Migration: do not cast `ConformanceReason` variants to integers with `as`; discriminants shifted when variants were removed, so match on the variant or use an explicit mapping.
- `enum_no_repr_variant_discriminant_changed` `ConformanceReason::PropositionLimit`: Migration: do not cast `ConformanceReason` variants to integers with `as`; discriminants shifted when variants were removed, so match on the variant or use an explicit mapping.
- `enum_no_repr_variant_discriminant_changed` `ConformanceReason::RewrittenBinding`: Migration: do not cast `ConformanceReason` variants to integers with `as`; discriminants shifted when variants were removed, so match on the variant or use an explicit mapping.
- `enum_no_repr_variant_discriminant_changed` `ConformanceReason::TraceDomainLimit`: Migration: do not cast `ConformanceReason` variants to integers with `as`; discriminants shifted when variants were removed, so match on the variant or use an explicit mapping.
- `enum_no_repr_variant_discriminant_changed` `ConformanceReason::UnsupportedProfile`: Migration: do not cast `ConformanceReason` variants to integers with `as`; discriminants shifted when variants were removed, so match on the variant or use an explicit mapping.
- `enum_variant_missing` `ConformanceReason::InvalidInput`: Migration: delete the match arm for `ConformanceReason::InvalidInput`; formula and catalog documents are validated at construction, so no input produces this variant.
- `pub_module_level_const_missing` `PGM01_POLICY_REVISION`: Migration: delete uses; the constant named a policy revision no code consumed.
- `pub_module_level_const_missing` `TL_MLTL_REVISION`: Migration: delete uses; the tl-mltl revision is recorded in `Cargo.toml` only, and reports no longer carry it.
- `pub_module_level_const_missing` `TL_SYNTAX_REVISION`: Migration: delete uses; the pinned tl-syntax revision is recorded in `Cargo.toml` only, and records no longer carry it.
- `struct_pub_field_missing` `ConformanceReport::evaluator_revision`: Migration: delete reads and struct-literal initializers of the field; serialized conformance reports no longer carry `evaluatorRevision` (see Removed).
- `struct_pub_field_missing` `ConformanceReport::syntax_revision`: Migration: delete reads and struct-literal initializers of the field; serialized records no longer carry `syntaxRevision` (see Removed).
- `struct_pub_field_missing` `PastConformanceReport::evaluator_revision`: Migration: delete reads and struct-literal initializers of the field; serialized conformance reports no longer carry `evaluatorRevision` (see Removed).
- `struct_pub_field_missing` `PastConformanceReport::syntax_revision`: Migration: delete reads and struct-literal initializers of the field; serialized records no longer carry `syntaxRevision` (see Removed).
- `struct_pub_field_missing` `RewriteReport::syntax_revision`: Migration: delete reads and struct-literal initializers of the field; serialized records no longer carry `syntaxRevision` (see Removed).

## 0.3.0

`tl-rewrite` is the last of the four MLTL crates (`tl-syntax`, `tl-parse`,
`tl-mltl`, `tl-rewrite`) in this first coordinated release. The version skips
0.2.0 so that all four crates share one version number past `tl-mltl`'s
existing v0.2.0. Changes are relative to v0.1.0.

### Added

- **Past-time (history) rewrites.** The new `past_catalog()` is an immutable
  catalog (`tl-rewrite.past-catalog/v1`). It admits only
  `mltl.origin-complete-history/v1` and reuses the reviewed
  profile-independent Boolean rules. It adds exactly two canonical folds that
  the specification states: `O[1,1] p` to strong Previous, and the expanded
  Boolean dual of Since to primitive Triggered. `rewrite` and `replay` accept
  formula-v2 past-profile documents. A rewrite preserves the formula version,
  semantic profile, caller formula identity, source revision, source spans and
  replay bindings.
- **Past equivalence.** `check_past_equivalence` compares two
  `PastEvaluationContext`s. Each context pairs a formula with the exact history,
  anchor and proposition-map identity it is evaluated against. The comparison
  runs through `tl_mltl::past::evaluate_past` over event-position and exact
  fixed-sample histories and returns a `PastConformanceReport` /
  `PastConformanceReason`. A refusal from the evaluator, including a resource
  limit, stays a typed non-conclusive result and never becomes a verdict.
- **Profile-separated rewrite architecture.** The engine is split into one
  subsystem per semantic profile, under the public modules `catalog`, `engine`,
  `equivalence`, `replay` and `report`. The crate-root re-exports are unchanged.
  A formula is dispatched only to the subsystem for its own profile. Online,
  mixed, unknown and structurally invalid inputs are refused without a partial
  output.
- **Strict record readers.** `RecordLimits`, `RecordReadError` and
  `RecordReadErrorCode` bound and type the admission of serialized reports.
- **W/M lowering parity (FR-008).** The engine sees only the 12 canonical
  tl-syntax node kinds. Weak-until `W[a,b]` and strong-release `M[a,b]` reach it
  only as the primitive graphs tl-syntax lowers them to. Tests establish that a
  lowered graph and the same graph built by hand produce identical rewrite,
  refusal, budget-exhaustion and bounded-conformance reports. The same holds
  for contextual rewrite with complete and incomplete signal catalogs, and for
  clean-ascii/v2 text parsed by `tl-parse`. Nine lowering mutants each fail
  parity. A source inspection confirms that `src/` has no derived-operator
  branch.
- **CI gate-set binding (NFR-004).** `make guarded-ci` runs the `ci_guard`
  entry point, a Rust program outside Make. Before it starts Make, it refuses a
  Makefile or calling environment that could suppress a failed prerequisite.
  When Make returns, it checks the completion records against the declared
  `ci` gate set, whatever exit code Make reported. Each gate receives its own
  per-run token, so a recipe cannot write the completion record for a
  different gate. The README, `CLAUDE.md` and the hosted workflow all invoke
  this entry point, and TC-065 checks that they do.

### Changed

- **Dependencies.** The historical dev-only aliases
  `tl-parse-derived`, `tl-syntax-lowering` and `tl-syntax-parser-seam` are
  gone. The W/M parity and parser-seam tests now run against the released
  crates.
- **Typed signal and requirement context in rewrite evidence.** The
  contextual APIs from v0.1.0 (`rewrite_with_context`, `replay_with_context`
  and `check_equivalence_with_context`) now carry tl-syntax 0.3.0's typed
  `SignalCatalogDocument` and `RequirementContextDocument` owner contracts into
  the report identities (`signal_catalog_sha256`, `requirement_context`,
  `binding_failure`). The past subsystem preserves replay bindings through each
  fold. An unresolved binding is still reported as
  `RewriteStatus::UnresolvedBinding` or as a typed conformance reason and is
  never rewritten silently.
- **The vendored corpus is removed.** The shared past-history corpus is read
  from the compiled dependency through `tl_syntax::CORPUS_DIR` rather than
  from an in-repo copy.
- **No Quire-ecosystem dependency.** The production and dev
  dependency graphs carry no `quire-observation` or other AGPL component.

### Breaking changes

- **The formula and signal types are those of tl-syntax 0.3.0.** Every public
  function takes and returns `tl_syntax` types (`FormulaDocument`,
  `SignalCatalogDocument`, `RequirementContextDocument`, `SourceSpan`), and
  those types come from the compiled tl-syntax revision. A consumer that compiles
  a different tl-syntax revision gets distinct, incompatible types.
  *Migration:* pass documents across the boundary as canonical
  formula-v1/v2 JSON bytes and read them with
  `FormulaDocument::from_json_bytes`.
- **Inputs can carry the past-time vocabulary.** tl-syntax 0.3.0 adds
  `SemanticProfile::OriginCompleteHistoryV1`, formula-v2 and five past
  `NodeKind`s, none of which v0.1.0's tl-syntax could represent. `rewrite`
  dispatches such a document to the past subsystem.
  *Migration:* handle the new tl-syntax variants in your own `match`es (see the
  tl-syntax 0.3.0 changelog). If a caller must stay future-only, check the
  document's semantic profile before calling `rewrite`.
