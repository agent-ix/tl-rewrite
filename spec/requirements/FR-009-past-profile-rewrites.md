---
id: FR-009
title: "Rewrite origin-complete past-profile formulas"
type: FR
relationships:
  - target: ix://agent-ix/tl-rewrite/StR-001
    type: implements
  - target: ix://agent-ix/tl-rewrite/StR-002
    type: implements
  - target: ix://agent-ix/tl-rewrite/FR-001
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/FR-002
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/FR-003
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/FR-007
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-011
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-013
    type: depends_on
---

# FR-009: Rewrite origin-complete past-profile formulas

## Description

When a validated formula names `mltl.origin-complete-history/v1`, the
tl-rewrite library shall traverse its complete O/H/Y/S/T graph and apply only catalogued
equivalences admitted for that exact profile.

## Inputs

- A validated `tl-syntax.formula/v2` document whose profile is exactly
  `mltl.origin-complete-history/v1` and whose graph contains only atoms,
  Boolean nodes, and the closed O/H/Y/S/T vocabulary.
- The existing rewrite options, bounded resource limits, caller formula
  identity, engine source revision, and optional validated signal catalog and
  requirement context.
- Exact production revisions of `tl-syntax` and `tl-mltl` that expose the
  accepted graph/profile and origin-complete evaluator contracts.

## Outputs

- A deterministic context-free or contextual rewrite report carrying the past
  catalog digest and preserving formula-v2, semantic profile, caller/source
  identities, source spans, and replay inputs.
- A fixed-point formula only for `unchanged` or `normalized`; typed
  `invalid_input`, `unsupported_profile`, `unresolved_binding`,
  `budget_exhausted`, or `non_convergent` outcomes carry no successful output.
- A separate immutable `past_catalog()` document. The existing `catalog()`
  remains the closed-trace future catalog with its prior rule bytes and order.

## Behavior

The past catalog SHALL:

1. reuse B01–B24 only, with their existing identities, revisions, priority,
   preconditions, and Boolean derivations, while changing their catalogued
   semantic profile to `mltl.origin-complete-history/v1`;
2. normalize `Once[1,1] p` to primitive `StrongPrevious p`, as stated by
   `ix://agent-ix/tl-syntax/FR-011-AC-5`;
3. normalize `not((not p) Since[a,b] (not q))` to primitive
   `p Triggered[a,b] q`, as stated by
   `ix://agent-ix/tl-syntax/FR-011-AC-3`; and
4. leave H, Y, S, T, every other O interval, every partial dual shape, and any
   other unproved past transformation semantically and structurally unchanged
   except for independently admitted Boolean rewrites inside their operands.

Traversal, compaction, interning, budgets, step ordering, rolling digests,
context binding, and replay use the existing deterministic engine. Rebuilding
a formula preserves the input schema version instead of down-converting v2 to
v1. A past request and its conformance refusal name the past catalog; the
future-only bounded-equivalence API returns a non-conclusive
`unsupported_profile` result rather than presenting future lookahead evidence
as proof over histories.

Past semantic checks compare original and rewritten documents with the pinned
`tl-mltl` origin-complete evaluator over event-position and exact fixed-sample
histories, every generated anchor, interval boundaries, pre-origin extension,
and Boolean valuations. They do not introduce a second production evaluator
or qualify a monitor.

## Error and Boundary Conditions

- Formula-v2 validation rejects future/past mixtures, profile-incompatible
  nodes, invalid topology, unknown profile values, and unknown node values
  before rewrite execution.
- `mltl.online-prefix/v1` remains unsupported. An unsupported or invalid
  profile cannot select an executable past rule by catalog fallback.
- An exact past pattern either produces its complete replacement and trace step
  or the attempt fails under the existing checked budget with no successful
  output; no partial transformed graph escapes.
- The existing `u32` node identities and interval bounds, `u64` work and
  application counters, checked node ceiling, and maximum-iteration policy
  remain in force. The admitted past folds do not expand the graph.
- Contextual requests bind all surviving propositions against the supplied
  catalog before a successful output escapes; changed or omitted identity
  inputs make replay mismatch.

## Compatibility and Dependency Policy

Formula-v1 bytes and future rewrite semantics remain unchanged. Revision-bearing
report bytes advance when this repository deliberately advances the exact
compiled `tl-syntax` or `tl-mltl` revisions; the report schemas, fields, status
meanings, and deterministic identical-call guarantee do not change. TC-035
pins the new candidate baseline and the provenance gate requires every reported
revision to equal the manifest and lockfile revision that actually ran.

The canonical cross-repository past/history corpus does not yet exist at this
task boundary. Its publication, checksum, and exact parser/evaluator/rewriter
consumer replay are allocated to `ix://agent-ix/tl-syntax/Task-005` under
`tl-syntax#54`, which depends on this implementation. Until that successor
lands, this repository uses local constructed fixtures and generated histories
and makes no shared-corpus completion claim.

No rewrite path executes a parser, network service, callback, plugin, foreign
monitor, Quire, Quoin, or qualification workflow. All input strings are opaque
reported identities bounded by their owning upstream types; no unsafe code or
side-effecting extension point is introduced.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-009-AC-1 | The past catalog has a distinct schema/version/digest, contains exactly B01–B24 plus the two stated past folds under only `mltl.origin-complete-history/v1`, and leaves the legacy future catalog byte-identical. | Test (TC-046) |
| FR-009-AC-2 | O/H/Y/S/T operands are traversed and rebuilt as formula-v2; only `O[1,1]` and the complete Triggered dual fold, while all unproved and partial past shapes remain unchanged and source/profile/caller identities are preserved. | Test (TC-047) |
| FR-009-AC-3 | Every reused Boolean rule is exercised under the past profile and the original/output verdicts agree for every Boolean valuation under both admitted clock models. | Test (TC-048) |
| FR-009-AC-4 | Context-free and contextual past reports round-trip and replay exactly, while independently changing input, catalog, options, trace steps, output, or signal catalog produces mismatch. | Test (TC-049) |
| FR-009-AC-5 | Online, mixed, unknown, invalid, unresolved, exhausted, and unproved cases remain distinct fail-closed or unchanged outcomes; no future-only conformance result is presented as past equivalence evidence. | Test (TC-050, TC-051) |
| FR-009-AC-6 | Generated event-position and exact fixed-sample histories preserve both admitted folds at every anchor and boundary, exact dependency attribution passes, and the v1 report-byte baseline changes only for the reviewed dependency advance. | Test (TC-030, TC-035, TC-052) |

## Dependencies

Depends on FR-001 catalog identity, FR-002 bounded execution, FR-003 trace and
replay identity, FR-007 contextual reports, the accepted central past
semantics/profile requirements, and merged `tl-syntax#53` and `tl-mltl#63`
implementations. The shared-corpus successor is `tl-syntax#54`; native Quire
projection and correspondence remain owned by `quire-contract-ir#70` and
`quire-contract-ir#71`.
