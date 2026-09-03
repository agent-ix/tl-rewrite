---
id: FR-006
title: Adopt the shared assurance intake path
type: FR
relationships:
  - target: ix://agent-ix/tl-rewrite/StR-002
    type: implements
---

# FR-006: Adopt the shared assurance intake path

## Description

The repository shall produce its catalog, rewrite, replay, bounded-equivalence,
counterexample and provenance results with its own tools in declared structured
formats, and shall obtain every static specification fact from Quire and every
retention, integrity, audit and receipt behaviour from Quoin, without either
tool executing a producer and without a repository-local generic evidence
framework.

## Behavior

- Component versions are classified by the compatibility matrix packaged with
  the pinned Engineering Assurance release. This repository observes what is
  installed and restates no version rule of its own.
- One target, `make assurance-inputs`, runs the producers and writes their
  structured results. Everything downstream consumes those files and refuses to
  create them; an absent input is an error naming that target, never a skip.
- Each proof attestation states the verdict read out of the bytes its producer
  wrote. No verdict is inferred from a transcript, an exit code alone, or a
  caller's expectation.
- A semantic-equivalence counterexample is retained as a witness: the
  disagreeing trace, both verdicts, and an independent replay of that trace
  against both documents. It is never reduced to a boolean.
- Bounded agreement is reported as bounded. Every equivalence row carries the
  limitation naming the enumerated domain, and no gate promotes it into a
  universal proof or into qualification of a consuming tool.
- An excluded catalog rule is reported as unsupported and a document that will
  not decode is reported as malformed. Neither fails its proof obligation and
  neither is transcribed as a plain pass.
- Twelve verification outcomes remain distinguishable across the intake path,
  each demonstrated by a case that produced it, and each negative paired with a
  positive control observed to be accepted.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-006-AC-1 | The adopted component versions are classified by the packaged Engineering Assurance compatibility matrix, not by a local restatement of it, and no component resolves from the internal mirror. | Test (TC-023) |
| FR-006-AC-2 | Native rule-conformance, counterexample, normalization and provenance results are produced by this repository's tools in a declared structured format and transcribed by Quoin without Quoin or Quire executing the producer. | Test (TC-024) |
| FR-006-AC-3 | Static specification, obligation, and coverage facts come from a Quire export that names every requirement in the repository, and Quire executes no producer. | Test (TC-025) |
| FR-006-AC-5 | Pass, fail, unavailable, unsupported, inconclusive, not-computed, malformed, partial, stale, suspect, vacuous, and tampered remain twelve distinguishable states, each demonstrated and each negative paired with a positive control. | Test (TC-027) |
| FR-006-AC-6 | Every retained counterexample carries its disagreeing trace and both verdicts, is replayed against both documents outside the enumeration that found it, the witness count agrees with the counterexample corpus rather than with the producer, and the witnesses survive into the bytes Quoin retained. | Test (TC-028) |
| FR-006-AC-7 | No repository-local generic runner, evidence envelope, manifest, identity framework, retention store, audit store, anchor verifier, retraction registry, evidence schema, retained-evidence directory, compatibility view, or aggregate verdict remains in the repository. The removal census enumerates tracked and untracked-not-ignored repository paths through Git, independently constrains its exact denials, tracked areas and declaration exemptions, scans raw bytes, and fails closed with a named diagnostic when it cannot read a path. | Test (TC-029) |

## Dependencies

Depends on the released Engineering Assurance, quire-cli, Quoin and ix-flow
pins recorded in `assurance/pins.json`, and on FR-001 through FR-004 for the
domain behaviour whose results it transcribes.
