---
id: NFR-076
title: Keep the default build and dependency tree unchanged by the solver feature
type: NFR
quality_attribute: maintainability
relationships:
  - target: ix://agent-ix/tl-rewrite/FR-091
    type: constrains
---

# NFR-076: Keep the default build and dependency tree unchanged by the solver feature

## Statement

The tl-rewrite crate shall resolve, in its default feature set, exactly the
normal and build dependency tree that it resolved before the `solver` feature
existed, and every repository gate shall run for both the default and the
`solver` configurations.

## Scope

`Cargo.toml`, the `cargo tree` output of the default and `--no-default-features`
configurations, `deny.toml`, and the declared `ci` prerequisite set of NFR-004.
The committed lockfile necessarily lists the optional tl-solve packages, which
is disclosed and not counted as a tree change.

## Rationale

TL-405 requires that the default dependency tree of tl-rewrite be unchanged, and
cargo-deny runs over the whole lockfile, so the supply-chain gates must cover
the optional packages too.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Lines of difference between `cargo tree -e normal,build --locked` for the default set and the recorded pre-feature baseline | 0 | 0 | Test |
| Gates in the declared `ci` set that skip the `solver` configuration | 0 | 0 | Inspection |

## Verification

A test records the default tree at the commit before the feature (the
baseline file `tools/default-tree.baseline`) and compares it byte for byte with
the current output. The Makefile's `ci` prerequisites include a `test-solver`
gate, and NFR-004's reconciliation lists it.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-076-AC-1 | The default `cargo tree -e normal,build --locked` output equals the recorded baseline byte for byte, and the `solver` tree differs from it only by tl-solve and its own dependencies. | integration-testing (TC-980) |
| NFR-076-AC-2 | `make deny` passes on the default and the all-features graphs with `deny.toml` adding only the tl-solve repository to `allow-git`, `make ci` declares a gate that runs the `solver` test suite, and NFR-004's declared-versus-executed reconciliation includes it. | integration-testing (TC-981) |

## Dependencies

Constrains FR-091; follows NFR-004 for the gate set.
