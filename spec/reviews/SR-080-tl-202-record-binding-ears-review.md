---
id: SR-080
title: "ears-conformance review of NFR-004-AC-9 (TL-202 per-gate record binding)"
type: SpecReview
analysis: ears-conformance
scope: "spec/requirements/NFR-004-gate-set-integrity.md"
review_set: subset
---

## Summary

Ran `quire validate --scope . "spec/**/*.md" --summary` (zero `[ears:*]`
warnings anywhere in `NFR-004-gate-set-integrity.md`, including the new
Scope item 4, the Rationale addition, and AC-9) and applied the semantic
judgment the engine cannot make to AC-9 and its supporting prose. One
wording precision finding, already fixed in the reviewed text rather than
left open.

## Findings

| ID      | Severity | Summary                          | Refs   |
| ------- | -------- | --------------------------------- | ------ |
| FND-001 | medium   | AC-9 originally read "a fresh, unpredictable token" — verifiable by test only as "does not collide / is not a formula of public inputs," not as cryptographic unpredictability, and sat in mild tension with the Scope residual paragraph's own explicit disclaimer that the token "is not a secret in the cryptographic sense." Reworded to "not derivable from `CI_GUARD_RUN_ID` and a gate's own name alone," matching exactly what `mint_gate_tokens`' doc comment claims and what TC-064/the unit tests actually check, and matching the Scope paragraph's own framing so the two no longer read as pulling in different directions. Applied to both AC-9 and Scope item 4. | NFR-004-AC-9 |
| FND-002 | low      | AC-9 is one compound sentence (minting+delivery, then the write-gate condition, joined by "so"), not a single atomic `shall`. Consistent with this NFR's existing style — AC-5's single sentence is similarly compound (missing/removed/backdated, three conditions, one obligation) — and TC-064 already covers it as one coherent integration behavior. Not split; splitting would add an AC/TC pair for no gain in what gets tested. No change. | NFR-004-AC-9, NFR-004-AC-5 |
| FND-003 | low      | AC-9 and the Scope/Rationale prose use declarative "mints"/"delivers"/"is written only when" rather than EARS `shall`, matching every other AC in this document (none of AC-1 through AC-8 use `shall` either) and the Statement section's own two `shall` sentences, which are the ones EARS trigger-word scanning actually targets. Consistent with the document's established archetype dialect; not a defect. No change. | NFR-004-AC-9 |

