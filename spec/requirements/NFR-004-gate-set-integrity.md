---
id: NFR-004
title: Bind the declared and executed CI gate set
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-006
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/NFR-003
    type: extends
  - target: ix://agent-ix/tl-rewrite/StR-001
    type: traces_to
---

# NFR-004: Bind the declared and executed CI gate set

## Statement

If any prerequisite declared on the `ci` target does not both run its own
recipe to completion and report success from that recipe's own outcome, then
the CI entry point shall report `ci` as failed, naming every such
prerequisite. While invoking Make, the CI entry point shall refuse to proceed
if the Makefile text or the invocation environment it is about to hand to
Make carries an execution-control state capable of suppressing
prerequisite-failure propagation.

The CI entry point is a dedicated program distinct from and outside the Make
process it drives, invoked in place of a bare `make ci` by every local and
hosted caller.

## Scope

This requirement owns a single artifact: a CI entry point that is not itself
a Make recipe, invoked in place of a bare `make ci` by every local and hosted
caller. It owns that entry point's three checks, and the tests that exercise
each:

1. **Static inspection** of the Makefile text before delegating to Make, for
   every execution-control surface the removed parse-time guard named:
   `SHELL`, `.SHELLFLAGS`, `MAKEFLAGS`, `.ONESHELL:`, `.DEFAULT:`, `.IGNORE:`,
   `.SILENT:`, a `-`-prefixed recipe line, `$(eval …)`, and `include` —
   together with two recipe-content patterns the original guard did not
   police: a recipe line containing `|| true`, and one redirecting stderr to
   `/dev/null`. An `include`d file is scanned by the same check rather than
   exempted; `$(eval` is refused outright rather than partially analyzed.
2. **Invocation-environment control**: the entry point does not forward an
   inherited `MAKEFLAGS` to the Make process it starts, and invokes Make with
   an explicit, minimal flag set rather than trusting the caller's shell
   environment — closing the vector where `-i`/`-k`/`-S`-equivalent behavior
   is requested through the environment rather than through Makefile text,
   which static inspection alone cannot see.
3. **Declared/executed reconciliation** after Make returns: each `ci`
   prerequisite's recipe writes a per-gate completion record naming itself
   and its own recipe's exit status, written only on that recipe's own
   successful completion; the entry point compares the resulting record set
   against the exact declared `ci` prerequisite set and reports a violation
   naming any mismatch in either direction, rather than trusting Make's own
   exit code.
4. **Per-gate record binding**: the entry point mints a fresh token, not
   derivable from `CI_GUARD_RUN_ID` and a gate's own name alone, for every
   declared gate immediately before invoking Make, and
   delivers each gate's token into only that gate's own recipe environment
   through Make's target-specific-variable scoping — not through the
   top-level environment the entry point also hands Make, where every
   recipe and every subprocess it spawns already shares the run identifier
   item 3's completion record carries. A completion record is written only
   when the caller presents the token minted for the gate it names; a
   subprocess of a *different* declared gate's own recipe — a compromised or
   buggy `cargo test`, `cargo clippy`, python script, or external
   `quire`/`quoin` invocation, the concrete risk Linear TL-202 raised — no
   longer has enough information, by inheritance alone, to write a
   convincing completion record for a gate other than its own.

It does not own the correctness of any individual gate's recipe (formatting,
lint, corpus, conformance, and the rest each remain owned by the requirement
that specifies that gate's behavior), and it does not own producer-input
integrity, which NFR-003 and the Quoin-bound `assurance-inputs` chain already
establish by deriving attested results from a producer's own written bytes.
Those two mechanisms are complementary rather than overlapping: the digest
chain notices a producer that never ran by finding its output absent, and
this requirement depends on that chain remaining intact so it need not
duplicate coverage of the four gates re-run inside `assurance-inputs`. It is
blind, however, to a gate — `fmt-check`, `lint`, `test`, `check-corpus`,
`deny`, `audit-unsafe`, `rustdoc`, and the `quire validate` half of `spec` —
whose recipe writes nothing the chain reads, and whose failure Make's own
execution controls can suppress before it is ever observed. This requirement
closes exactly that residual, and NFR-003's Scope section records the handoff
rather than continuing to describe it as unowned.

The binding this requirement establishes holds only for invocations that go
through the entry point. `make ci` remains directly invocable as a Makefile
target for local convenience; it is not itself the assured gate once this
requirement is implemented, and nothing mechanical stops a person from typing
it directly and asserting a result without the entry point's report. This
requirement closes that gap for every *documented and hosted* path rather
than for every possible human action: every reference to running the full
local gate set — this repository's README, `CLAUDE.md`, and any hosted
workflow dispatch — names the entry point, not a bare `make ci`, so the
ordinary and automated paths cannot bypass the binding by using the
pre-existing alias. A person who deliberately runs `make ci` directly and
reports its exit code without the entry point is outside what an
in-repository control can prevent; that residual is disclosed here rather
than claimed away, matching this repository's existing convention of stating
what a control does not reach.

The mechanism is repository-local and domain-specific to this Makefile and
this `ci` target; it is not a proposal for a shared, cross-repository
control. A future generalization of the same guarantee into Quoin or
Engineering Assurance is a separate, later decision and is out of scope here.

Item 4's binding is deliberately scoped to what Make's own execution model
can actually enforce, and what it cannot is disclosed here rather than
claimed away. Every recipe in a single `ci_guard ci` invocation — and every
subprocess any of them spawns — runs as the same OS user with unrestricted
read access to the same filesystem; Make provides no sandboxing or process
identity between one gate's recipe and another's. The per-gate token is
therefore not a secret in the cryptographic sense: it withholds a gate's
token from a process that only *inherits* environment state the way it
already inherits the run identifier (the entire mechanism TL-202 reported as
missing), but it does not withhold that token from a process that
deliberately locates and reads the entry point's own token store off disk,
which nothing in this control encrypts or otherwise protects from a reader
already inside that shared trust domain. Closing that further residual would
require running each gate's recipe as a genuinely separate OS principal —
e.g. a distinct container or user per gate with its own credential the
others cannot read even via the filesystem — which is a materially larger
architectural change than a completion-record protocol, is not proposed
here, and is out of scope for the reasons the paragraph above already gives
for not generalizing this mechanism further. What item 4 closes is the
inheritance-only forgery TL-202 described as exploitable "in principle" by
any of the thirteen gates' own subprocesses with zero additional effort;
what it does not close is a subprocess that goes looking for the token store
specifically, which was already a materially higher-effort, more deliberate
action than the one this control removes.

## Rationale

Make's failure-propagation behavior is not an invariant of the tool; it is a
default that a single misplaced directive, recipe prefix, dynamically
evaluated fragment, included file, or inherited flag overrides for the entire
run, silently and without changing which targets appear to exist. A reviewer
reading `ci:`'s prerequisite list, or a person trusting a green `make ci`, is
trusting that nothing in the file or its invocation environment has done
that — a property Make itself has no mechanism to assert about its own
execution, and a property a check that only reads Makefile text cannot fully
assert either, since some of these overrides arrive through the environment
rather than the file. Binding what was declared to what actually executed,
from a vantage point outside Make and with the entry point controlling both
the text it delegates to and the environment it delegates with, converts that
trust into a checked property: the entry point either observes thirteen
genuine passes or it does not report a pass at all, regardless of which
mechanism — present or future, textual, dynamic, or environmental — caused a
gate not to run its own work.

Item 3's completion record is bound to a *run* (NFR-004-AC-5) but, until
NFR-004-AC-9, not to a *gate*: `CI_GUARD_RUN_ID` is exported once into the
top-level Make process and every recipe and subprocess in that run's process
tree inherits it identically, so a completion record for any declared gate
was writable by any of the thirteen gates' own recipes, or anything they
shell out to, using nothing but a value already in their environment and a
gate name already public in the Makefile. Reconciliation (item 3) is not an
independent catch for this: `reconcile` only asks whether *some* record
naming a gate exists for this run id, so a record forged for a gate before
that gate's own (genuinely failing) recipe runs is indistinguishable, to
reconciliation, from a real one — reconciliation is deceived into treating
the affected gate as present, not tipped off. What still catches the overall
run today is the trailing raw Make exit-status check, and only because every
currently-known way to hide a gate's *own* genuine failure from Make's exit
code is already blocked by item 1's static scan; that check cannot say
*which* gate misbehaved, only that Make exited non-zero, so a forged record
degrades the entry point's most useful output — reconciliation naming the
specific gate — into the coarse, undifferentiated signal `NFR-004`'s own
Rationale above says a checked property should not still be depending on.
Binding the record to a token Make's own target-specific-variable scoping
places only in the one recipe environment that token was minted for turns
"which run wrote this" into "which run *and which gate's own recipe* wrote
this", so reconciliation itself is no longer deceivable this way and keeps
naming the actual gate.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Execution-control surfaces recognized as failure-propagation risks (`SHELL`, `.SHELLFLAGS`, `MAKEFLAGS`, `.ONESHELL`, `.DEFAULT`, `.IGNORE`, `.SILENT`, `-`-prefixed recipe line, `$(eval …)`, `include`, `\|\| true`, stderr-to-null redirect) | 12/12 | 12/12 | Test |
| `ci` invocations through the entry point where the declared prerequisite set and the executed-completion-record set are reconciled before a result is reported | 100% | 100% | Test |
| Induced-failure classes from the tracked reproduction (`.IGNORE:` prepended; every recipe replaced by a failing stub; a backdated or removed completion record) rejected by the entry point | 3/3 | 3/3 | Test |
| False rejections of an unmodified, passing Makefile and a clean invocation environment | 0 | 0 | Test |
| Documented or hosted invocation paths for the full local gate set that still name a bare `make ci` instead of the entry point | 0 | 0 | Inspection |
| A gate's own recipe (or a subprocess it spawns) attempting to write a completion record for a *different* declared gate, presenting only `CI_GUARD_RUN_ID` and that gate's public name | 0/1 accepted | 0/1 accepted | Test |

## Verification

A positive-control run against the unmodified Makefile, a clean invocation
environment, and every gate passing demonstrates the entry point reports
success and does not over-trigger. Negative controls reproduce the tracked
measurement directly: a Makefile copy with a single `.IGNORE:` line
prepended; a skeleton Makefile whose recipes are all replaced by a failing
command; and, separately, a completion-record set that is removed or
backdated for one gate while that gate remains declared as a `ci`
prerequisite. Each must be rejected by the entry point even though a bare
`make ci` on the same file would exit non-zero for the wrong reason, exit
zero with the directive present, or exit zero on a stale record the raw exit
code never inspects. A further control sets `MAKEFLAGS` in the calling
environment to a value equivalent to `-i`/`-k` before invoking the entry
point, without touching the Makefile text, and must also be rejected — this
demonstrates the environment-control check independently of the static-text
check, so a failure in one is attributable without the other masking it. A
fifth and sixth reproduction, of the Linear TL-202 report directly. Fifth: a
fixture Makefile whose declared `ci` prerequisites are `gate-a` alone, where
`gate-a`'s recipe, after recording its own completion, also attempts
`ci_guard record gate-b` for an undeclared gate using only the shared
`CI_GUARD_RUN_ID` — must be refused for lacking `gate-b`'s own token, and
must produce no completion record naming `gate-b` at all, while `gate-a`'s
own legitimate record is unaffected. Sixth, and the sharper case the
Rationale above describes: `ci: gate-a gate-b`, where `gate-a` genuinely
succeeds and, as a side effect Make itself does not check the exit status
of (`$(shell ...)` evaluated while expanding a recipe line, standing in for
a subprocess `gate-a`'s own recipe spawned whose failure does not propagate
to that recipe's exit code — e.g. an unawaited child of `cargo test`),
attempts to forge `gate-b`'s record before `gate-b`'s own recipe — which
genuinely fails — ever runs. Must be refused; reconciliation must go on to
name `gate-b` itself as missing, not fall back to the raw Make exit-status
check's undifferentiated failure. An inspection pass confirms the README,
`CLAUDE.md`, and any hosted workflow dispatch reference the entry point
rather than a bare `make ci`.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-004-AC-1 | The entry point parses the Makefile text, including any `include`d file, before invoking Make, and refuses to proceed if `SHELL`, `.SHELLFLAGS`, or `MAKEFLAGS` is assigned, if `.ONESHELL:`, `.DEFAULT:`, `.IGNORE:`, or `.SILENT:` appears as a special target, if any recipe line under a `ci` prerequisite is `-`-prefixed or contains `\|\| true` or a stderr-to-`/dev/null` redirect, or if `$(eval` appears anywhere in the text. | Test |
| NFR-004-AC-2 | The entry point does not forward an inherited `MAKEFLAGS` to the Make process it starts and invokes Make with an explicit, minimal flag set; an invocation whose calling environment sets `MAKEFLAGS` to an `-i`/`-k`/`-S`-equivalent value is refused before Make runs. | Test |
| NFR-004-AC-3 | Each `ci` prerequisite's recipe writes a per-gate completion record naming the gate and its own recipe's exit status only on that recipe's own successful completion. | Test |
| NFR-004-AC-4 | After Make returns, the entry point reconciles the set of completion records against the exact declared `ci` prerequisite set from the Makefile and reports a violation naming every declared gate without a record and every record without a declared gate, rather than trusting Make's own exit code. | Test |
| NFR-004-AC-5 | A completion record that is missing, removed after being written, or backdated to a revision or timestamp that precedes the run under evaluation is treated as no record for that gate, and the run is reported as a violation rather than a pass. | Test |
| NFR-004-AC-6 | Reproducing the tracked measurement — a `.IGNORE:`-prepended Makefile copy, and a skeleton Makefile with every recipe replaced by a failing stub — against the entry point yields a non-zero exit and a named violation, not a reported pass. | Test |
| NFR-004-AC-7 | An unmodified Makefile, a clean invocation environment, and every gate genuinely passing yields a zero exit from the entry point with no violation reported. | Test |
| NFR-004-AC-8 | The repository's README, `CLAUDE.md`, and any hosted workflow dispatch that runs the full local gate set invoke the entry point rather than a bare `make ci`. | Inspection |
| NFR-004-AC-9 | The entry point mints a fresh per-declared-gate token before invoking Make, not derivable from `CI_GUARD_RUN_ID` and a gate's own name alone, and delivers each gate's token into only that gate's own recipe environment; a completion record is written only when the caller presents the token minted for the gate it names, so a call presenting `CI_GUARD_RUN_ID` and a different declared gate's name, but not that gate's own token, is refused and writes no record. | Test |
