---
id: SR-074
title: "failure-domain review of NFR-004 (TL-64 gate-set-integrity remediation)"
type: SpecReview
analysis: failure-domain
scope: "spec/requirements/NFR-004-gate-set-integrity.md"
review_set: all
---

## Summary

Asked what still defeats this control after it is implemented as specified.
Found one significant unstated failure mode in the first draft (the entry
point is a new program that must actually be invoked — nothing stopped a
caller from continuing to run `make ci` directly and bypassing the whole
guarantee) and closed it in the Scope and a new AC. Also checked the
execution-control surface list itself against the removed guard's original
scope and found it incomplete; closed by broadening the static-inspection
list and adding an environment-control check.

## Findings

| ID      | Severity | Summary                          | Refs   |
| ------- | -------- | --------------------------------- | ------ |
| FND-001 | high     | First draft specified the entry point's checks but never addressed the trivial bypass: since `ci:` remains a normal Makefile target, any caller — human or hosted workflow — can simply keep running `make ci` directly and never invoke the new entry point at all, silently receiving zero protection while believing the repository's disclosed gap was closed. Closed by adding a Scope paragraph requiring every *documented and hosted* invocation path (README, CLAUDE.md, hosted workflow dispatch) to name the entry point instead of bare `make ci`, plus AC-8 (Inspection) to check it, and an explicit disclosure that a person who deliberately types `make ci` directly is a residual this control cannot mechanically prevent — stated rather than hidden, matching NFR-003's own "Qualification Boundary" convention. | NFR-004 |
| FND-002 | high     | The original AC-1's execution-control list (`.IGNORE`, `.SILENT`, `.ONESHELL`, `.DEFAULT`, `-`-prefix, `\|\| true`, stderr-to-null) omitted several surfaces the *removed* guard explicitly policed per the Makefile's own header comment and the ticket text: `SHELL`, `.SHELLFLAGS`, `MAKEFLAGS`, `$(eval …)`, and `include`. A control that remediates "the guard that used to police X" but polices a strict subset of X is a real completeness gap, not a style issue. Closed by expanding AC-1 to the full named class and adding recursive scanning of `include`d files and an outright refusal on any `$(eval` occurrence. | NFR-004 |
| FND-003 | medium   | `MAKEFLAGS` can carry `-i`/`-k`/`-S`-equivalent behavior through the calling *environment* rather than the Makefile text, which no amount of static text inspection can see — a caller could leave the Makefile untouched and still neuter propagation via an inherited environment variable. Closed by adding an explicit invocation-environment-control check (AC-2: the entry point does not forward inherited `MAKEFLAGS` and invokes Make with an explicit flag set) and a corresponding negative control in Verification. | NFR-004 |
| FND-004 | low      | Considered whether the entry point's own binary/source could itself be tampered with to fake success (turtles-all-the-way-down). Not addressed mechanically — this is the same trust-root question Make itself has, and the repository's existing pattern (NFR-003's Qualification Boundary: "branch protection and remote review... establish resistance") already answers it at the process layer, not the tool layer, for every control in this repository. No new requirement text added; noted as consistent with existing scope decisions rather than a gap unique to NFR-004. | NFR-004 |
