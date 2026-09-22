---
id: SR-082
title: "failure-domain review of NFR-004-AC-9 (TL-202 per-gate record binding)"
type: SpecReview
analysis: failure-domain
scope: "spec/requirements/NFR-004-gate-set-integrity.md"
review_set: subset
---

## Summary

Walked the four failure-domain checks (extension-point failure behavior,
entity identity, evaluation purity, topological robustness) against AC-9's
mechanism. The one genuinely new extension point — token-store write
failure — already fails closed by construction; the rest either reuse an
identity/purity pattern this NFR already establishes or do not apply.

## Findings

| ID      | Severity | Summary                          | Refs   |
| ------- | -------- | --------------------------------- | ------ |
| FND-001 | low      | Extension point: `write_gate_tokens` (disk I/O immediately before invoking Make) can fail (permissions, disk full). Implementation (`run_ci`) treats that failure as **strict**: it refuses to invoke Make at all and reports `ExitCode::FAILURE`, the same fail-closed default AC-9's sibling `reset_gates_dir` failure already uses one call earlier in the same function, and the same default `write_record`/`is_valid_gate_name` already apply to untrusted-shaped input elsewhere in this module. AC-9's wording does not spell this failure mode out explicitly, but neither does AC-3 spell out `write_record`'s own I/O-failure behavior, and this NFR's existing granularity does not itemize every internal I/O failure as a separate AC. Consistent with the document's established level of detail; no new gap introduced by AC-9 specifically. No change. | NFR-004-AC-9 |
| FND-002 | low      | Entity identity: the per-gate token store is keyed by gate name, the same key `GateRecord`/`read_records`/`write_record` already use — no new or ambiguous identity concept introduced. No change. | NFR-004-AC-9 |
| FND-003 | low      | Evaluation purity: token minting reads `/dev/urandom`, wall-clock time, and pid — side effects, but this is generator/nonce logic, not user-supplied guard/filter/validator logic the purity check is aimed at (no simulation or replay requirement applies to it), and its non-determinism is the entire point (documented in `mint_gate_tokens`' own doc comment, including the fallback when `/dev/urandom` is unavailable). Not applicable. No change. | NFR-004-AC-9 |
| FND-004 | low      | Topological robustness: not applicable — AC-9 introduces no graph/tree traversal or search. No change. | NFR-004-AC-9 |

