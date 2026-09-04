---
id: FR-007
title: Bind shared temporal context into rewrite reports
type: FR
relationships:
  - target: ix://agent-ix/tl-rewrite/StR-003
    type: implements
  - target: ix://agent-ix/tl-syntax/FR-007
    type: depends_on
---

# FR-007: Bind shared temporal context into rewrite reports

## Description

When a caller supplies the shared tl-syntax signal catalog and optional
requirement context, tl-rewrite shall validate formula bindings and carry that
context through deterministic rewrite, replay, and bounded-equivalence reports
without changing the existing context-free API or its v1 wire bytes.

## Inputs

- A validated `tl-syntax` formula document and the existing rewrite or
  conformance request fields.
- One validated `tl-syntax.signal-catalog/v1` document.
- Either no caller context or one complete validated
  `tl-syntax.requirement-context/v1` document.

## Behavior

- Context-aware entry points use the shared tl-syntax document and borrowed
  validation APIs. They define no rewrite-local signal, domain, binding,
  requirement, clause, anchor, or source-context model.
- Each contextual native report records a SHA-256 identity of the complete,
  deterministically serialized signal-catalog document and embeds the exact
  optional shared requirement-context document. The request digest is
  domain-separated from v1 and binds the complete catalog document, explicit
  presence or absence of context, formula identity and bytes, options, rule
  catalog, and applicable source/dependency identities.
- Context-aware rewrite validates every input proposition before executing the
  rewrite and every successful output proposition before returning success.
  The same catalog is used at both boundaries. Rules may remove proposition
  occurrences as part of a reviewed semantic rewrite, but they do not project,
  rename, retype, or mutate catalog entries; every surviving proposition keeps
  its original binding.
- A first unresolved input or output proposition is an explicit typed
  non-success that names the formula locus and proposition identity. It is not
  converted into an empty catalog, an untyped signal, a context-free result, or
  a successful output.
- Context-aware replay re-executes the contextual request. Editing, omitting, or
  substituting the supplied catalog or context changes the observed native
  report and produces a typed mismatch.
- Context-aware bounded equivalence validates both formula documents against
  the same supplied catalog before enumeration. Binding refusal remains
  non-conclusive and distinct from semantic mismatch, evaluator error, and
  resource limits.
- Requirement-context clause spans and existing `RewriteStep` formula-node
  spans are separate fields with separate loci. Neither is substituted for the
  other.

## Versioning and compatibility

- Existing `rewrite`, `replay`, and `check_equivalence` entry points retain
  their current behavior and remain context-free wrappers over the common
  implementation. Their native records continue to serialize byte-for-byte as
  `tl-rewrite.report/v1`, `tl-rewrite.replay/v1`, and
  `tl-rewrite.conformance/v1` for identical inputs.
- Context-aware entry points emit closed v2 forms of those same native report
  families. V1 rejects contextual fields; v2 requires a signal-catalog identity
  and represents context absence explicitly. Missing required v2 identity,
  unknown fields, unsupported versions, or an invalid field combination is
  rejected during decoding.
- The v2 forms are tl-rewrite domain results, not a generic evidence envelope.
  Quoin may ingest their serialized producer output through the existing shared
  assurance path, but tl-rewrite imports or executes no Quoin, Quire,
  Engineering Assurance, contract-IR, parser, evaluator replacement, runner,
  collector, or retention runtime.
- The crate manifest remains `publish = false` and `MIT OR Apache-2.0`; this
  work makes no release, qualification, accreditation, or certification claim.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-007-AC-1 | A contextual rewrite report carries the exact optional shared requirement id, revision, clause id, anchor and clause span, the complete signal-catalog digest identity, and the existing ordered per-step node spans at their distinct locus. | Test (TC-031) |
| FR-007-AC-2 | Exact contextual replay verifies, while independently changing, dropping, or substituting any catalog declaration, domain, name, binding, context field, context presence, formula, option, rule-catalog identity, or intermediate makes replay return mismatch. | Test (TC-032) |
| FR-007-AC-3 | Contextual rewrite validates input and successful output formulas against one shared catalog, preserves every surviving proposition binding, and reports the first unresolved input or output proposition as a typed non-success without successful output. | Test (TC-033) |
| FR-007-AC-4 | Contextual bounded equivalence carries the same context identities and refuses an unresolved original or rewritten proposition as a distinct non-conclusive reason before enumeration. | Test (TC-034) |
| FR-007-AC-5 | Existing context-free APIs return the same semantic results and exact v1 serialized bytes; contextual v2 records round-trip strictly and reject v1/v2 field smuggling, missing identities, unknown fields, and unsupported versions. | Test (TC-035) |
| FR-007-AC-6 | Contextual native results serialize as producer-owned domain records usable by the existing Quoin intake without Quoin or Quire executing the producer; no local generic evidence or execution machinery is added, and the crate remains unpublished under its dual license. | Test (TC-036) |

## Dependencies

Depends on FR-002 through FR-005 and the exact shared types and wire documents
introduced by `agent-ix/tl-syntax#15` under `tl-syntax` FR-007. The future
contract-IR/FRETish adapter remains owned by `agent-ix/quire-contract-ir#57`.
