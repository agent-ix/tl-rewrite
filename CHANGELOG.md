# Changelog

All notable user-visible changes to `tl-rewrite` are recorded here. The crate is
distributed as a git source release (`publish = false`); versions are git tags.

## 0.3.0

`tl-rewrite` is the last of the four MLTL crates (`tl-syntax`, `tl-parse`,
`tl-mltl`, `tl-rewrite`) in this first coordinated release. The version skips
0.2.0 so that all four crates share one version number past `tl-mltl`'s
existing v0.2.0. Changes are relative to v0.1.0.

Release gate: the full local gate (`make guarded-ci`) passes at these pins, and
`ci_guard` reconciled all 13 declared `ci` gates. The gates are fmt-check,
lint, test, check-corpus, conformance, counterexamples, normalization, deny,
audit-unsafe, spec, msrv, rustdoc and assurance. `make spec` passes strict
`quire validate` and strict `quire coverage`, which backs all 144 rows, so the
owner's 0.3.0 exception for unbacked rows was not needed for this crate.

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

- **Dependencies are pinned to the 0.3.0 release tags.** `tl-syntax` is
  `=0.3.0` at `4a5614193d21e5ae99950ae683b04ba0ec931358` and `tl-mltl` is
  `=0.3.0` at `452f013a3168512603d427bce3360bc14c1175a6`. The dev-only
  `tl-parse` is `=0.3.0` at `496020aad5595b870f141b88995cdc2ea6a5994e`.
  `TL_SYNTAX_REVISION` and `TL_MLTL_REVISION` name these revisions, and
  `scripts/check_provenance.py` requires them to match `Cargo.toml` and
  `Cargo.lock`. All of them compile one tl-syntax, so `Cargo.lock` resolves
  exactly one `tl-syntax` and one `tl-parse`. The historical dev-only aliases
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
  from an in-repo copy. The corpus's pinned manifest digest is now the one at
  tl-syntax v0.3.0. That release rewrote two sentences of the corpus README and
  left the cases unchanged.
- **No Quire-ecosystem dependency.** Since tl-mltl 0.2.0 the production and dev
  dependency graphs carry no `quire-observation` or other AGPL component.
- **The shared assurance lane is on engineering-assurance v0.2.1 and ix-flow
  0.2.3** (previously v0.2.0 and 0.0.4). v0.2.1 is the release that carries the
  recorded human acceptance of its compatibility matrix, so the
  `UNKNOWN-ea-acceptance-not-in-a-release` unknown is resolved.
- **MSRV is now Rust 1.98.1** (1.75 at v0.1.0). `rust-toolchain.toml` pins that
  exact toolchain.

### Breaking changes

- **The formula and signal types are those of tl-syntax 0.3.0.** Every public
  function takes and returns `tl_syntax` types (`FormulaDocument`,
  `SignalCatalogDocument`, `RequirementContextDocument`, `SourceSpan`), and
  those types come from the pinned tl-syntax revision. A consumer that compiles
  a different tl-syntax revision gets distinct, incompatible types.
  *Migration:* pin `tl-syntax` (and `tl-mltl`, if you use it) to the same v0.3.0
  tags. Alternatively, pass documents across the boundary as canonical
  formula-v1/v2 JSON bytes and read them with
  `FormulaDocument::from_json_bytes`.
- **Report bytes and identities differ from v0.1.0.** `RewriteReport`,
  `ReplayReport` and `ConformanceReport` record the compiled tl-syntax and
  tl-mltl revisions (`syntax_revision`, `evaluator_revision`). A report's own
  digest therefore moves with those revisions, and so does the report digest
  that a `ReplayReport` embeds. Schema versions (`tl-rewrite.report/v1`,
  `tl-rewrite.replay/v1`, `tl-rewrite.conformance/v1`) and status meanings are
  unchanged. *Migration:* re-derive any stored report digest with 0.3.0. Do not
  compare digests of reports produced by different crate versions.
- **Inputs can carry the past-time vocabulary.** tl-syntax 0.3.0 adds
  `SemanticProfile::OriginCompleteHistoryV1`, formula-v2 and five past
  `NodeKind`s, none of which v0.1.0's tl-syntax could represent. `rewrite`
  dispatches such a document to the past subsystem.
  *Migration:* handle the new tl-syntax variants in your own `match`es (see the
  tl-syntax 0.3.0 changelog). If a caller must stay future-only, check the
  document's semantic profile before calling `rewrite`.
- **Rust 1.75 through 1.98.0 can no longer build the crate.** *Migration:*
  build with Rust 1.98.1 or newer and raise your own `rust-version` to match.
