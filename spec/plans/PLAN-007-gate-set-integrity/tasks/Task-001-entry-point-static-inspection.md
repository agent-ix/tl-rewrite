---
id: Task-001
title: "Entry-point scaffold + static Makefile inspection"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-rewrite/NFR-004
    type: references
  - target: ix://agent-ix/tl-rewrite/TC-057
    type: verifies
---
# Task-001: Entry-point scaffold + static Makefile inspection

## Scope

Add a new Rust CI entry point — a binary or example, matching the existing
`examples/rule_conformance.rs`-style producer pattern already used in this
repository — that will replace bare `make ci` for every local and hosted
caller once this plan finishes. This task delivers the entry point's
skeleton and its first check: static inspection of the Makefile text (and
any `include`d file, scanned recursively) for the execution-control surfaces
NFR-004-AC-1 names.

## Subtasks

- [ ] Scaffold the entry point binary with a `ci` subcommand (or default
  behavior); no other targets are in scope for this plan.
- [ ] Parse the Makefile text for: `SHELL`/`.SHELLFLAGS`/`MAKEFLAGS`
  assignment, `.ONESHELL:`, `.DEFAULT:`, `.IGNORE:`, `.SILENT:`, a
  `-`-prefixed recipe line under a `ci` prerequisite, `|| true`, a
  stderr-to-`/dev/null` redirect, and `$(eval`.
- [ ] Recursively scan any `include`d file for the same eleven surfaces;
  refuse if it cannot be resolved/read.
- [ ] On any match, refuse to invoke Make at all and report a violation
  naming the surface and its location.
- [ ] Define and document the completion-record file contract (path
  convention under e.g. a `target/ci-gates/` directory, gate-name field,
  exit-status field) that Task-003 will write to and Task-004 will read —
  this is a design deliverable of this task, not deferred to Task-004.
- [ ] Add fixture Makefiles covering: a clean control, and one instance of
  each of the eleven surfaces individually (including the `include` case).

## Deliverables

- The entry-point binary/example with the static-inspection check wired in
  and refusing to proceed on any positive match.
- A short contract note (doc comment or `docs/` note) specifying the
  completion-record file format.
- TC-057 test fixtures and the test itself, green.

## Notes

- This is domain-specific, repository-local logic per NFR-004's Scope — not
  a proposal for a shared cross-repo control. Do not reach for
  `scripts/*.py` or extend the shared-assurance Python lane; this is new
  Rust, per the owner directive in Linear TL-64.
- The eleven-surface list is deliberately the *union* of what the removed
  parse-time guard used to police (`SHELL`, `.SHELLFLAGS`, `MAKEFLAGS`,
  `.ONESHELL`, `.DEFAULT`, `.IGNORE`, `.SILENT`, `-`-prefix, `$(eval)`,
  `include`) plus two recipe-content patterns it did not (`|| true`,
  stderr-to-null) — see SR-074/FND-002. Do not silently narrow this list; a
  narrower list is the exact defect this plan exists to fix.
- `$(eval` is refused outright rather than partially analyzed — do not try
  to evaluate what it would produce.
- Unblocks: Task-002 (env control extends this binary), Task-003 and Task-004
  (build against the completion-record contract this task fixes), Task-005
  (references the entry point by name).
