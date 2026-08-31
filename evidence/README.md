# Retained evidence

Run `bash scripts/collect_evidence.sh` from a clean repository root. Each run
creates a revision-and-UTC-time-scoped directory, refuses overwrite, and
retains stdout, stderr, exit status, identities, limitations, a canonical
`quire.derivation-evidence/v1` envelope, and an external SHA-256 file.

Set `PGM01_SCHEMA` and `PGM01_VALIDATOR` to the exact merged PGM-01 Draft 7
schema and validator. Missing external gates are recorded as unavailable,
never successful. Inputs pin tl-syntax, tl-mltl, WEST, the rule catalog,
parameters, dependencies, schemas, corpus, and PGM-01.

The collector architecture is adapted under MIT OR Apache-2.0 from the
same-program tl-mltl collector at revision
`a9b7847199c1d846abd7b67901cd6836374ccee2`, tailored to catalog, rewrite,
replay, bounded-equivalence, and WEST gates.

Evidence informs a pending human source-release decision. It neither proves
arbitrary rewrite schemas nor approves, validates, qualifies, accredits,
certifies, or publishes a consuming project.
