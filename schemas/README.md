# Retained evidence schemas

These two files are what the local evidence framework validated against. That
framework is gone: `make ci` runs no JSON-Schema validation, no script imports
`jsonschema`, and nothing in this repository validates any document against
either file. Retention, integrity checking, audit, attestation and receipt are
owned upstream by Quoin, whose packaged FR-063/FR-064/FR-065 schemas are the
shapes the shared assurance path uses.

The two files are kept rather than deleted, but for two *different* reasons, and
the difference was measured rather than assumed. Writing "both are frozen because
the retained envelopes name them" would have been false for one of them.

| File | SHA-256 | Status |
|---|---|---|
| `tl-rewrite-evidence-manifest-v1.schema.json` | `859ebdd66869023808e88a89e09969731170606692137a56772e5dbc43bf31b0` | **Frozen.** All 11 retained envelopes reference `tl-rewrite.evidence-manifest v1` by exactly this digest. The committed bytes *are* the bytes those immutable records name. |
| `tl-rewrite-evidence-input-v1.schema.json` | `5bda9d5f4fafc1910859e28d64c254be546398a54a1168e2c902cde8df28f7ce` | **Retained, and not the referenced artifact.** The retained envelopes reference three distinct `tl-rewrite.evidence-input v1` digests and the committed file matches none of them. |

## Why the input schema is a fourth revision

The input schema was tightened twice during the v0.1 review rounds, after the
records that reference it were sealed. Every referenced version is recoverable
from Git history:

| Referenced digest | Recoverable at |
|---|---|
| `6cd06977e9bfbd1c51e913c3fdb2e8377cfe28cd0f5e26b6bcdbb206d9839bb0` | `9d28279332efdfc6cb70ecd66f0b8a2171c8b3aa` |
| `49ba18db114f4f00742de559bd99fcdf7de2e131e6c4942b6b038d7905236ae1` | `a889382df7fd6ef116f4ca18186c21562feadefa` |
| `1d8fc991fadac11bc60459728978fe4f2d92b3aee3125ccd1980aec386e689e3` | `1b08a6c9e7bca9ebd2ff3831dfc5db9a5749dee4` |

Git history and pull-request review are the integrity boundary this repository
has always declared for retained bytes, so those revisions are the record, and
the file here is kept only so the family's final shape stays readable next to the
records it grew out of.

Both digests are stated in `assurance/pins.json`, re-derived by
`scripts/check_shared_pins.py` on every `make pins`, and pinned again by
`tests/shared_assurance.rs`. A silent edit to either file is a reported
difference in two independent places.

Nothing else in the repository may reference these files by name. A test asserts
that over a census of ten directories walked recursively — `scripts`, `tests`,
`examples`, `src`, `spec`, `docs`, `.github`, `assurance`, `corpus`, `schemas` —
plus every file at the repository root, discovered rather than listed. Only this
README, `assurance/pins.json`, `assurance/change-assurance.json` and the test
itself may name them.
