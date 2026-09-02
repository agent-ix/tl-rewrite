---
id: MRS-001
title: tl-rewrite v0.1 master requirements
type: MasterRequirements
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: depends_on
  - target: ix://agent-ix/tl-syntax/MRS-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/MRS-001
    type: depends_on
---

# Master Requirements Specification

## Purpose

This specification defines deterministic, bounded, semantics-preserving MLTL
rewriting. It owns a provenance-bearing rule catalog, controlled execution,
replayable traces, and bounded equivalence evidence against the exact tl-mltl
reference evaluator and permitted WEST cases.

PGM-01 governs compatibility, provenance, evidence, human authority, and
qualification boundaries. Formula/profile identities come from the exact
tl-syntax revision; semantic comparison comes from the exact tl-mltl revision
that `Cargo.toml`, `Cargo.lock` and `src/lib.rs` all resolve, which
`scripts/check_provenance.py` requires to agree on every run.

That revision used to be named by retained evidence instead. It is not any more:
issue #13 deleted the retained records under the authority of
`agent-ix/engineering-assurance#7`, and a specification that still sourced an
identity from them would name an authority this repository no longer has. The
resolved dependency graph was already the enforced authority — the retained
records named `da2c7704`, a revision the build had not used since the pin moved,
which is the defect `TC-030` exists to catch.

## Scope

### In Scope

- Stable identities, revisions, preconditions, profiles, and provenance for
  enabled and deliberately excluded rules.
- Deterministic bottom-up rewriting with explicit iteration, node,
  application, and logical-work budgets.
- Versioned success and non-conclusive reports with replayable step traces.
- Exhaustive bounded closed-trace comparison and a pinned WEST fixture subset.

### Out of Scope

- Text parsing, a second formula AST, or a general-purpose optimizer.
- Unbounded equality saturation or wall-clock performance claims.
- Production monitoring, proof-system certification, or automatic release.
- Treating bounded differential evidence as a universal proof for new rules.

## System Overview

tl-rewrite consumes validated tl-syntax graphs. The catalog and rewrite engine
are deterministic library modules. Trace verification replays the exact input,
catalog, strategy, and budgets. Equivalence validation enumerates a declared
finite trace domain and invokes tl-mltl without embedding a second evaluator.

## Requirements Architecture

FR-001 owns the rule catalog, FR-002 bounded execution, FR-003 trace/replay,
FR-004 equivalence and WEST evidence, FR-005 the versioned interchange boundary,
and FR-006 the shared assurance intake path. NFR-001 constrains
determinism/resources, NFR-002 constrains provenance and qualification claims,
and NFR-003 owns the qualification controls.

FR-005 owned the PGM-01 evidence boundary as well until issue #13 deleted the
retained archive; that allocation is removed rather than reassigned, because no
requirement owns an evidence boundary this repository no longer has.

## References

- [tl-rewrite epic](https://github.com/agent-ix/tl-rewrite/issues/6).
- [WEST research artifacts](https://temporallogic.org/research/WEST/).
- [WEST canonical repository](https://github.com/zwang271/WEST).
- [PGM-01](https://github.com/agent-ix/quire-contract-ir/blob/main/spec/program/PGM-01-governance.md).
