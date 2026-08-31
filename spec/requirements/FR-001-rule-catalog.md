---
id: FR-001
title: Define a provenance-bearing rewrite catalog
type: FR
relationships:
  - target: ix://agent-ix/tl-rewrite/StR-001
    type: implements
---

# FR-001: Define a provenance-bearing rewrite catalog

## Description

The library shall expose a deterministic catalog whose rules carry stable
identities, revisions, class, supported profiles, preconditions, disposition,
and either a stated derivation or primary published source.

## Behavior

- Enabled v1 rules are limited to Boolean identities, negation duals, temporal
  singleton/constant identities, and directly derived Until/Release cases.
- Enabled v1 rules apply only to `mltl.closed-trace/v1`; online-prefix
  applicability remains excluded until its own conformance population exists.
- The catalog distinguishes normalization, simplification, negation, and
  temporal transformations.
- Disputed, unbounded, or growth-sensitive candidates remain explicitly
  excluded rather than partially implemented.
- The canonical catalog digest changes when any rule metadata changes.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-001-AC-1 | Every enabled and excluded rule has a unique stable identity, revision, class, profile set, precondition, disposition, and provenance reference. | Test (TC-001, TC-002) |
| FR-001-AC-2 | Enabled rule outcomes match their stated finite-trace derivations on primitive and nested fixtures. | Test (TC-003) |
| FR-001-AC-3 | Any rule metadata or ordering change produces a different canonical catalog digest and invalidates stale traces. | Test (TC-004) |

## Dependencies

Depends on exact tl-syntax profiles and PGM-01 provenance requirements.
