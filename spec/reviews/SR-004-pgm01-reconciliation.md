---
id: SR-004
title: tl-rewrite merged PGM-01 reconciliation
type: SpecReview
analysis: base
scope: PGM-01 requirements and the tl-rewrite v0.1 candidate
review_set: all
---

# tl-rewrite merged PGM-01 reconciliation

## Summary

Merged PGM-01 policy: `agent-ix/quire-contract-ir#12` at
`7dac9d8c19952412b56a0347387666e2ca81e01d`.

Envelope schema: `quire.derivation-evidence/v1`, SHA-256
`0946e235e9e4b0fa79e9b9ec27ae157b303c17de0a9408d3cc04968fb7152256`.

The collector architecture is adapted under MIT OR Apache-2.0 from the
same-program tl-mltl collector at immutable revision
`a9b7847199c1d846abd7b67901cd6836374ccee2`. The adaptation records catalog,
rewrite, replay, bounded equivalence, WEST provenance/license, complete command
failures, canonical validation, overwrite refusal, and external checksums.
Human review remains pending.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-401 | medium | PGM-01 is merged and exactly reconciled; independent human review and the exact source-release decision remain open. | PGM-01, AP-001, AA-001 |

## Policy mapping

| Policy requirement | tl-rewrite disposition | Evidence or remaining gate |
|---|---|---|
| PGM-01-R01 schema compatibility | Catalog, rewrite, replay, conformance, local evidence input, and manifest records use explicit closed v1 identities and reject unknown fields. | FR-003, FR-005; serde and evidence schemas |
| PGM-01-R02 exact pins | Candidate evidence records source, merged policy/schema, toolchain, syntax/evaluator dependencies, catalog source, WEST source/license, parameters, inputs, and outputs. | Collection input, manifest, canonical envelope |
| PGM-01-R03 release order | tl-rewrite pins exact reviewed tl-syntax and tl-mltl source candidates; downstream repositories must pin the eventual human-reviewed tl-rewrite revision. | Cargo.toml, Cargo.lock; human decision remains open |
| PGM-01-R04 licensing and provenance | Crate, schemas, collector adaptation, and derived fixtures are MIT OR Apache-2.0; copied WEST bytes retain the byte-identical upstream MIT license. | Cargo.toml, corpus/README.md, corpus checksums, evidence/README.md |
| PGM-01-R05 clean-room boundary | The crate consumes typed graphs and does not add a text grammar, parser table, or copied grammar implementation. | MRS-001 and repository inspection |
| PGM-01-R06 human authority | Agent-produced rules, implementation, and evidence remain separate from independent human review and release. | AP-001, AA-001, envelope provenance |
| PGM-01-R07 classification | tl-rewrite is a linked analysis component and requires consuming-project verification. | AP-001, CAC-001, README.md |
| PGM-01-R08 common envelope | Revision-scoped evidence emits every canonical field and is gated by the exact PGM-01 Draft 7 schema and validator. | Collector and retained validator outputs |
| PGM-01-R09 retention and decision | New runs refuse overwrite, retain stdout/stderr/status/digests/limitations, and record no automated release decision. | Collector, external checksum file, AA-001 |
| PGM-01-R10 qualification boundary | Catalog, replay, WEST, and bounded-equivalence evidence confer no universal proof, monitor qualification, project validation, accreditation, or certification. | AP-001, AA-001, README.md |

The merged policy requires no additional public semantic API. Independent
review, protected-branch checks, and the human source-release decision remain
external workflow gates.
