---
id: NFR-003
title: Make qualification controls explicit and fail closed
type: NFR
quality_attribute: reliability
---

# NFR-003: Make qualification controls explicit and fail closed

## Statement

Candidate qualification shall keep the producer boundary observable, derive every
attested result from the bytes a producer wrote, keep the twelve verification
outcomes distinguishable, retain every semantic-equivalence counterexample as a
replayable witness, and grant no release authority.

This requirement previously also owned a control over the retained evidence
archive. Issue #13 deleted that archive under the authority of
`agent-ix/engineering-assurance#7`, and the control was removed with it rather
than restated over an empty set.

## Scope

This requirement owns the shared-assurance intake path: the pinned toolchain
declaration in `assurance/pins.json`, the change-assurance declaration in
`assurance/change-assurance.json`, the driver `scripts/assurance_chain.py`, the
pin classifier `scripts/check_shared_pins.py`, the provenance producer
`scripts/check_provenance.py`, and the tests that exercise them.

It no longer owns `tools.lock`, a local-CI runner, Make execution-control
probes, a collector, a finalizer, a manifest verifier, an evidence-profile
resolver, an anchor gate, or a retraction registry. Those were removed with the
local evidence framework they belonged to under issue #9. It no longer owns a
compatibility view, a retained evidence archive, or a frozen evidence schema
family either; those were deleted under issue #13.

That is a real reduction in local detection, and the extent of it is stated here
rather than minimised. Measured on this repository's own Makefile at the migration revision, not inherited from a sibling, and with the command named so the number can be re-derived. Run `make ci CARGO=false PYTHON=false QUIRE=false QUOIN=false ASSURANCE_DIR=target/ig-probe ASSURANCE_PYTHON=/bin/false` so that no gate's work happens: `make ci` exits **2** and stops at the first prerequisite. Prepend a single `.IGNORE:` line to the same Makefile and run the identical command: it exits **0** after **25** ignored recipe failures, and all 13 `ci` prerequisites report success. Eleven of them do so with their own recipe having failed; `assurance` is an aggregate whose two sub-targets each failed and were ignored; `audit-unsafe` invokes `bash` directly and is not reachable by the tool-variable sabotage, and a skeleton Makefile in which every recipe is replaced by `false` confirms it reports success too.
Nothing in this repository inspects
Make's own execution controls to notice.

A structural backstop exists but covers only part of the gate set. Quoin binds
each retained input by digest and every attested result is derived from the
producer's own bytes, so a *producer* that did not run yields an absent or empty
input the chain names. That protects the four targets whose work is re-run
inside `make assurance-inputs`. It does **not** protect a gate whose recipe
writes nothing the chain reads: `fmt-check`, `lint`, `test`, `check-corpus`,
`deny`, `audit-unsafe`, `rustdoc`, and the `quire validate` half of `spec` can
each be neutered and every remaining check stays green.

The residue is recorded as an open unknown in the change-assurance declaration
and tracked as `agent-ix/tl-rewrite#11`, which carries the reproduction.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Components classified by the packaged matrix | 4/4 | 4/4 | Test |
| Verification outcomes demonstrated and matched | 12/12 | 12/12 | Test |
| Negatives without an accepted positive control | 0 | 0 | Test |
| Attested results not derived from producer bytes | 0 | 0 | Test |
| Retained counterexamples without a replayed witness | 0 | 0 | Test |
| Automatic release decisions | 0 | 0 | Inspection |

## Verification

Behaviour tests invoke the gates rather than reimplementing them. The producer
boundary is asserted with two runs — producers replaced by logging stubs with the
log required to be empty, and a control that stubs the tool the chain does use
and requires the chain to fail — because an empty log and an unconsulted `PATH`
are otherwise the same observation. Mutation probes remove one load-bearing
check at a time and require the corresponding gate to go red.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-003-AC-1 | Every attested proof result is derived from the producer's own structured output; a producer whose output is absent, empty, or unreadable is an error naming the target that writes it, and never a pass. | Test (TC-024) |
| NFR-003-AC-2 | Neither Quire nor Quoin executes a producer, demonstrated by stubbing every producer and requiring no invocation, together with a control that stubs Quoin and requires the chain to fail. | Test (TC-024) |
| NFR-003-AC-3 | The twelve verification outcomes stay distinguishable, each demonstrated by a case that produced it and matched, with every negative paired with a positive control and a control naming a non-existent scenario refused. | Test (TC-027) |
| NFR-003-AC-5 | The revision constants this crate publishes as wire fields are the revisions Cargo.toml and Cargo.lock resolve, so a conformance report cannot attribute a verdict to a dependency that did not produce it. | Test (TC-030) |

## Qualification Boundary

These controls make a presented candidate and its retained artifacts
reproducible and reviewable. They confer no qualification, certification, or
accreditation. Bounded agreement over a horizon-complete domain is not a proof
of a rewrite schema, and nothing here qualifies a consuming monitor. Branch
protection and the remote review history, not the local repository, establish
resistance to history replacement.
