---
id: NFR-003
title: Fail closed on qualified-toolchain and evidence-profile drift
type: NFR
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---

# NFR-003: Fail closed on qualified-toolchain and evidence-profile drift

## Statement

Local qualification and retained evidence shall identify the tools that ran,
apply one explicit qualification profile, and positively corroborate that each
successful retained command produced its required evidence.

## Scope

Local CI preflight, evidence collection, retained outcome classification, and
the assurance-bound record are in scope.

## Rationale

An exit code alone cannot distinguish completed qualification work from a
substituted tool, a silently downgraded profile, or an empty transcript.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Unidentified qualification tools | 0 | 0 | Test |
| V2-era records with an absent or unknown profile | 0 | 0 | Test |
| Successful retained commands without positive corroboration | 0 | 0 | Test |

## Verification

The evidence contract and policy behavior tests redirect ambient tool paths,
remove or alter profile declarations, and empty or fabricate retained command
transcripts; every mutation must fail closed.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-003-AC-1 | Host-scoped qualification derives HOME, PATH, CARGO_TARGET_DIR, and RUSTUP_TOOLCHAIN from a source-locked manifest, verifies launcher, runtime, and Node executable identities, and re-derives active retained identities. Portable CI does not assert one host's lock. | Test (TC-022) |
| NFR-003-AC-2 | A retained record with an absent or unrecognized qualification profile is inconclusive unless it has a validated, record-digest-bound retraction; a v2-era omission and retraction of the assured record are rejected. | Test (TC-022) |
| NFR-003-AC-3 | Every non-silent successful v2 retained command has source-revision-applicable positive output, including source-derived Rust suite and exact passing-test counts; new diff-integrity records emit a positive completion signature. | Test (TC-022) |

## Dependencies

Applies PGM-01 to the local toolchain and retained evidence lifecycle.
