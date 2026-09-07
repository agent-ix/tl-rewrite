---
id: PLAN-003
title: Context-bound rewrite report implementation plan
type: Plan
status: in_progress
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-007
    type: references
  - target: ix://agent-ix/tl-rewrite/StR-003
    type: references
  - target: ix://agent-ix/tl-syntax/FR-007
    type: depends_on
---

# PLAN-003: Context-bound rewrite report implementation plan

## Objective

Implement `agent-ix/tl-rewrite#21` by consuming the exact shared tl-syntax
signal-catalog and requirement-context documents, binding them into native
rewrite/replay/equivalence results, and preserving all context-free v1 behavior
and bytes. The work extends domain code and the existing shared intake; it does
not add a generic evidence layer.

## Base and landing

The specification branch begins from merged tl-rewrite `main` revision
`f4a2e3ea22e19cfb7ebb669e4e9e1b0503dc5e57`. It is isolated from PR #20 at
`782c5c8108d950eaac2495004ae253dca726190f` as the issue requires.

Implementation is blocked until tl-syntax#15 lands. The published pre-merge API
at `0e6867aab3f21bbd3c64078257ed51d3bbda8d16` is sufficient for specification
and planning only. Once landed, this branch shall rebase or merge the then-
current tl-rewrite main as appropriate, pin the exact reviewed reachable
tl-syntax revision, and update `TL_SYNTAX_REVISION` and the lockfile together.
No local copy or temporary sibling type is permitted while waiting.

## Dependency DAG

```text
tl-syntax#14 reviewed landing
  -> tl-syntax#15 reviewed landing
    -> exact tl-syntax dependency/revision pin
      -> closed contextual v2 native report serialization
        -> shared context validation and domain-separated digest helpers
          -> contextual rewrite input/output binding
            -> contextual replay and mutation refusal
              -> contextual bounded-equivalence binding/refusal
                -> existing native producer + Quoin intake exercise
                  -> v1 snapshots, v2 strict-wire tests, full local gate
                    -> code review + gap analysis + downstream mltl#24 handoff
```

## Task File Mapping

| Task | Scope | Exit evidence |
|---|---|---|
| Task-001 | FR-007, StR-003, matrix, assurance impacts, composite review | Grammar-clean specification and SR-012 with no unresolved blocking finding |
| Task-002 | Exact shared pin, contextual v2 fields, strict version-dependent serialization, deterministic catalog/context identities | TC-031 and strict positive/negative wire controls without a local signal/context schema |
| Task-003 | Context-aware rewrite and replay entry points, input/output binding, contextual request digest, mutation matrix | TC-032 and TC-033, including exact replay and every independent substitution class |
| Task-004 | Context-aware bounded equivalence and existing native Quoin intake | TC-034 and TC-036 with no new generic runner, collector, adapter framework, or evidence envelope |
| Task-005 | Context-free snapshots, full verification, code review, gap analysis, downstream contract | TC-035, exact-head local gate, resolved findings, and mltl#24 handoff |

## Implementation shape

- Reuse `tl_syntax::SignalCatalogDocument`, `SignalCatalog`,
  `RequirementContextDocument`, and their validation/binding APIs directly.
  A valid shared catalog already excludes malformed and non-Boolean direct
  bindings; tl-rewrite adds only domain-specific input/output formula-locus
  refusal for a missing proposition binding.
- Factor rewrite and conformance execution behind internal context modes so the
  existing public context-free functions continue producing v1 records without
  fabricating an empty catalog or context. Add explicit context-aware entry
  points that require a shared catalog and accept optional shared context.
- Evolve the existing `RewriteReport`, `ReplayReport`, and
  `ConformanceReport` native families to admit closed v2 records. Custom
  version-aware serialization/deserialization must enforce: v1 has no
  contextual fields; v2 requires signal-catalog identity and a present context
  field whose value is a document or explicit `null`; unsupported versions and
  invalid combinations fail.
- Hash deterministic typed serialization with an operation- and version-specific
  domain separator. A contextual rewrite request binds the complete catalog,
  exact context presence/value, input, identity, options, rule catalog and
  engine/source identities. Conformance additionally binds both formulas,
  evaluator identity and conformance options.
- Bind the input before rewrite work and bind successful output before returning
  a success. A reviewed rule may remove proposition occurrences, but the shared
  catalog is never projected or mutated and every surviving proposition
  resolves to its unchanged binding. Conformance binds both operands before
  enumerating traces.
- Keep clause-level context spans and `RewriteStep` node spans in their own
  fields. Do not manufacture either from the other.
- Extend an existing native producer/intake declaration only as needed to prove
  that Quoin retains a contextual domain result without executing the producer.
  Do not create a new general producer protocol, evidence envelope, runner,
  collector, retention store, or local Quire/Quoin replacement.

## Verification method

TC-031 through TC-036 use native Rust unit/integration/snapshot tests and the
existing producer-owned shared assurance path. The mutation table changes one
catalog declaration, name, domain, binding, requirement id, revision, clause,
anchor, span, presence marker, formula, option, rule-catalog identity, or
intermediate at a time, with an exact accepted control beside each class.

V1 snapshots are captured before report-structure changes and assert exact JSON
bytes after them. V2 tests cover all three report families, exact context and
catalog identity, explicit `null`, missing-field refusal, unknown-field refusal,
unsupported-version refusal, and invalid v1/v2 combinations. No Python helper,
shell gate, Make target, bespoke assurance parser, or new hosted workflow is
introduced.

The full local `make ci CARGO_TARGET_DIR=target/cargo-review` gate runs only at
an exact candidate head after implementation and closing fixes. Hosted CI stays
manual-only and is not dispatched by this plan.

## Exit Criteria

1. The dependency and `TL_SYNTAX_REVISION` name the exact reviewed landed
   tl-syntax#15 commit; Cargo resolves one tl-syntax revision shared with
   tl-mltl.
2. Every FR-007 and StR-003 criterion has a named executing trace symbol, with
   planned matrix rows changed to implemented only after the tests run.
3. Context-aware rewrite, replay, and equivalence preserve the exact shared
   values, reject every specified substitution, and expose locus-specific
   unresolved-binding non-success without a successful output or enumeration.
4. Existing context-free API results and exact v1 serialized snapshots are
   unchanged; all contextual v2 forms are closed and deterministic.
5. A contextual native result traverses the existing producer-to-Quoin intake
   boundary without Quoin or Quire executing a producer and without new generic
   infrastructure.
6. The crate remains `publish = false` and `MIT OR Apache-2.0`; no report or
   assurance text claims provenance truth, universal equivalence, downstream
   qualification, accreditation, certification, or release.
7. The exact-head full local gate passes, code review and gap analysis contain
   no unresolved high or medium finding, and hosted CI was not dispatched.
