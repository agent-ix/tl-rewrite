---
id: SR-031
title: "Risk and complexity review of M0 assurance follow-ups"
type: SpecReview
analysis: risk-complexity
scope: "FR-006-AC-8 and NFR-003-AC-7 at 6a653b4"
review_set: all
---

## Summary

The tracked-file unwind path is medium technical risk because it combines
shared mutable state, panic unwinding, and cleanup. The workflow identity check
is low technical risk and medium volatility because it binds an external
package spelling and released version. Both have bounded mitigations.

## Risk Register

| Req | Tech Risk | Volatility | Drivers | Mitigation |
| --- | --- | --- | --- | --- |
| FR-006-AC-8 | Medium | Low | Shared tracked file, mutex poisoning, panic-time cleanup | RAII guard, explicit normal restore, forced spawn failure, forced restoration failure, exact-byte oracle |
| NFR-003-AC-7 | Low | Medium | npm package identity and workflow syntax may drift | Exact scoped pin, comment-safe all-token census, alias mutation, trigger check, runtime `--version` |

The top hazard is a panic leaving a tracked input modified; TC-038 is therefore
the critical-path gate. Hosted execution is not needed to test either change
and remains undispatched.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-3101 | low | No unmitigated high-risk or high-volatility item remains; preserve the restore-first order because it controls the only medium technical hazard. | FR-006-AC-8, NFR-003-AC-7 |
