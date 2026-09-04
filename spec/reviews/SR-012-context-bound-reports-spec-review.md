---
id: SR-012
title: Composite review of context-bound rewrite report specification
type: SpecReview
analysis: base
scope: "agent-ix/tl-rewrite#21; FR-007; StR-003; NFR-001; NFR-002; AD-001; CAC-001; AP-001; AA-001; test matrix"
review_set: all
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-007
    type: reviews
  - target: ix://agent-ix/tl-rewrite/StR-003
    type: reviews
---

# SR-012: Composite review of context-bound rewrite report specification

## Summary

Dependency, risk, evidence, integrity, scope, failure-domain, and EARS review
found no unresolved blocking ambiguity after six findings were corrected. The
design reuses the exact shared tl-syntax catalog and requirement-context
documents, extends the three existing native report families with closed v2
forms, and preserves the context-free v1 API and bytes. It does not introduce a
generic evidence envelope, an adapter-owned signal model, or another execution
and retention path.

The central compatibility decision is one report family per domain operation,
with schema-version-dependent fields, rather than a contextual wrapper around a
v1 result. A wrapper would create a second digest and status layer whose
relationship to the nested report could drift. The v2 form instead makes
context part of the native request identity and leaves existing v1 output
unchanged.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-1201 | high | An initial wrapper design would have created a parallel report envelope with a second status/request identity, contrary to the issue boundary and the shared assurance architecture. Contextual records are now closed v2 forms of the existing native report families. | FR-007 versioning, AD-001 |
| FND-1202 | high | Optional context could not be represented by an omitted optional wire field because omission would then be indistinguishable from deliberate absence, weakening replay. Every contextual v2 record now requires the field and encodes absence as explicit `null`. | FR-007 native report allocation, FR-007-AC-2, FR-007-AC-5 |
| FND-1203 | medium | “Incompatible binding” initially risked duplicating tl-syntax catalog validation or promising an unreachable local branch. A shared catalog document already refuses missing targets and non-Boolean direct bindings; tl-rewrite now owns only formula-to-valid-catalog resolution and its input/output locus. | FR-007 behavior, FR-007-AC-3, TC-033 |
| FND-1204 | medium | Requiring output to preserve every referenced proposition could prohibit valid rules such as `p AND false -> false`. The specification now permits semantic removal of occurrences while forbidding catalog projection or mutation and requiring every surviving proposition to keep the same binding. | FR-007 behavior, CAC-001 |
| FND-1205 | medium | A catalog digest in the report alone was insufficient to define replay input. The specification now requires the complete deterministic shared document as a domain-separated request-digest input and requires the caller to resupply it for replay; the report carries its SHA-256 identity rather than a copied catalog. | FR-007 native report allocation, FR-007-AC-1, FR-007-AC-2 |
| FND-1206 | low | Preserving opaque caller identities could be misread as validating their provenance. The requirements and assurance argument now limit the claim to deterministic byte identity and replay consistency. | StR-003 context, NFR-002-AC-3, AA-001 |

## Dispositions

| Finding | Disposition | Evidence |
|---|---|---|
| FND-1201 | **FIXED** | FR-007 allocates contextual data to v2 of rewrite, replay, and conformance; no generic or nested evidence envelope is permitted. |
| FND-1202 | **FIXED** | V2 requires an exact document or explicit `null`; decoding an omitted field is a refusal and TC-035 exercises that distinction. |
| FND-1203 | **FIXED** | FR-007 explicitly allocates catalog-shape/type errors to tl-syntax and locus-specific formula resolution to tl-rewrite. |
| FND-1204 | **FIXED** | FR-007 distinguishes removal of formula occurrences by a reviewed rule from mutation or projection of the supplied catalog. |
| FND-1205 | **FIXED** | Each contextual request digest consumes the complete catalog document, while the report records its digest identity and replay requires the document again. |
| FND-1206 | **FIXED** | StR-003, FR-007, NFR-002, and AA-001 deny provenance-truth inference. |

## Compatibility and failure-domain review

- Identical context-free calls must retain exact v1 JSON snapshots, not merely
  equal Rust results. V1 must reject v2 fields, and v2 must reject missing
  identity/context fields and unsupported versions.
- Context mutation probes cover every context field and presence; catalog
  probes independently cover declarations, names, domains, and bindings. A
  formula mutation and cross-formula substitution remain separate controls.
- Input binding is checked before rewrite work. Successful output binding is
  checked before a success report escapes. Conformance checks both operands
  before enumeration, so an unresolved proposition cannot consume a bounded
  domain and later appear as an evaluator error.
- Clause-level `RequirementContextDocument.source_span` is never copied into a
  `RewriteStep.source_span`; both remain visible and independently testable.
- The shared signal catalog may contain extra declarations and bindings for
  other formulas. A rewrite report binds the whole catalog, not a silently
  projected subset, so changes to an unused entry still change replay identity.

## Assurance and scope review

TC-036 shall exercise the existing producer-owned Quoin intake boundary with a
contextual native result. It may extend a domain producer or declaration, but it
may not create a local generic runner, collector, adapter framework, envelope,
retention store, or replacement for Quire, Quoin, or Engineering Assurance.
Hosted CI remains manual-only.

The crate remains `publish = false` and `MIT OR Apache-2.0`. Neither contextual
serialization nor bounded equivalence proves the truth of supplied provenance,
universally proves rewrite rules, qualifies a consuming monitor, or validates
the future contract-IR/FRETish adapter.

## Review conclusion

FR-007 and StR-003 are sufficiently bounded for planning. Implementation must
first consume the exact landed tl-syntax#15 revision; it must not copy types
from the published feature branch or stack onto tl-rewrite PR #20.
