# Retained evidence

Run `bash scripts/collect_evidence.sh` from a clean repository root. Each run
creates a revision-and-UTC-time-scoped directory, refuses overwrite, and
retains stdout, stderr, exit status, identities, limitations, a canonical
`quire.derivation-evidence/v1` envelope, and an external SHA-256 file.

`evidence/ANCHORS` binds every retained outer manifest. The complete local gate
requires an anchor for every record before checking the retained files,
manifest-to-artifact identities, and the summary re-derived from status files.
Retraction entries bind the exact outer-manifest digest, source revision, date,
reviewer, and supported reason. A documented legacy reseal is bound in the same
entry and checked against its Git introduction.

`make ci` is portable and does not require one workstation's `tools.lock` paths.
Evidence collection uses `make ci-for-evidence`, which additionally verifies
the declared host's launchers, Node runtime, and underlying Rust binaries. That
host-scoped result does not claim independent reproduction elsewhere.

Set `PGM01_SCHEMA` and `PGM01_VALIDATOR` to the exact merged PGM-01 Draft 7
schema and validator. Set `PGM01_PYTHON` when the validator's exact dependency
environment uses a different interpreter. Missing external gates are recorded
with exit status 125 as unavailable, never successful. The collector validates
the finalized envelope again, then writes a separate post-seal summary so the
envelope never self-attests. Inputs pin tl-syntax, tl-mltl, WEST, the rule
catalog, parameters, dependencies, schemas, corpus, and PGM-01. Passing records
also retain the resolved Python interpreter and active JSON-Schema format-checker
set; the assurance gate re-derives the source-bound parameter, collector, and
dependency digests.

The collector architecture is adapted under MIT OR Apache-2.0 from the
same-program tl-mltl collector at revision
`da2c7704a5347d063398c852acf6aa5bf9b5752d`, tailored to catalog, rewrite,
replay, bounded-equivalence, and WEST gates.

Evidence informs a pending human source-release decision. It neither proves
arbitrary rewrite schemas nor approves, validates, qualifies, accredits,
certifies, or publishes a consuming project.
