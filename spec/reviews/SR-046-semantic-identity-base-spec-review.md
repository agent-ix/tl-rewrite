---
id: SR-046
title: "Base specification review — semantic rewrite identity"
type: SpecReview
analysis: base
scope: "FR-002, TM-001"
review_set: base
---

# Base specification review — semantic rewrite identity

## Summary

The owner selected the base review set for the semantic interning and
formula-identity amendment. The review checked EARS grammar, identifier
integrity, matrix linkage, trace tags, scope containment, and the pre-existing
FR-007 v1 compatibility promise. The amendment preserves diagnostic spans
while requiring that they not affect structural interning, formula-level
identity, or cycle fingerprints. It explicitly records the resulting one-time
pre-release v0.1 byte-baseline correction rather than claiming that changed
digest values are byte-compatible with the defective baseline.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1501 | high | FIXED during rebase review: FR-002-AC-4 required span-free formula identities while FR-007 froze the earlier span-sensitive v1 bytes. Both could not be true for parser-shaped inputs. FR-007 now preserves the v1 API, fields, statuses, and schema, explicitly permits one pre-v0.1 digest-baseline correction, and freezes the corrected bytes in TC-035. | FR-002-AC-4, FR-007-AC-5, TC-035, TC-040 |
| FND-1502 | low | No remaining base-review defect found: FR-002-AC-4 resolves to fresh TC-040, which distinguishes parser-shaped span offsets while asserting both rule application and formula-level identities. | FR-002-AC-4, TC-040, tests/rewrite.rs |
