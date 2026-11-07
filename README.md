# tl-rewrite

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/k8DVhuYBR2)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)
[![Rust](https://img.shields.io/badge/rust-1.98.1%2B-orange?logo=rust)](Cargo.toml)

tl-rewrite simplifies a Mission-time Linear Temporal Logic (MLTL) formula to a
normal form with a catalog of semantics-preserving rules, and records what it did.
The report holds the rewritten document, every rule applied in order, and a digest
for every step.

Replay a report to confirm that it came from this catalog and these options. The
equivalence check shows that the original and the normal form agree on every
trace within the formula's horizon, or returns the first trace on which they
differ.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/hero-dark.svg">
  <img alt="A formula document goes into rewrite, which applies the 38 enabled rules of the catalog (2 more are excluded) bottom-up, first match, under explicit budgets, and returns a RewriteReport whose output is the normal form. replay re-runs the request and reports Verified or Mismatch. check_equivalence compares the original and the normal form on every trace within the horizon and reports Equivalent, or Mismatch with the first counterexample trace." src="docs/images/hero-light.svg">
</picture>

## Getting started

tl-rewrite is not on crates.io. Depend on it through git:

```toml
[dependencies]
tl-rewrite = { git = "https://github.com/agent-ix/tl-rewrite" }
```

Add `rev = "<commit>"` to pin a commit. It needs the Rust version in `rust-version` in `Cargo.toml`, or newer.

Enable the `infinite-trace` feature for the infinite-trace catalog: `features = ["infinite-trace"]`.

To build from a clone:

```sh
git clone https://github.com/agent-ix/tl-rewrite.git
cd tl-rewrite
cargo build --release
```

## Usage

This rewrites `not (not (true and p3))`, replays the report and checks the result
against the input.

```rust
use tl_rewrite::{
    check_equivalence, replay, rewrite, ConformanceOptions, ConformanceStatus, ReplayStatus,
    RewriteOptions, RewriteStatus,
};
use tl_syntax::{FormulaDocument, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

let nodes = vec![
    Node::new(NodeKind::True),
    Node::new(NodeKind::Proposition { proposition: PropositionId(3) }),
    Node::new(NodeKind::And { left: NodeId(0), right: NodeId(1) }),
    Node::new(NodeKind::Not { operand: NodeId(2) }),
    Node::new(NodeKind::Not { operand: NodeId(3) }),
];
let input = FormulaDocument::new(SemanticProfile::ClosedTraceV1, NodeId(4), nodes).unwrap();

let report = rewrite(&input, "demo", RewriteOptions::default(), "rev-1");
assert_eq!(report.status, RewriteStatus::Normalized);
let applied: Vec<&str> = report.steps.iter().map(|s| s.rule_id.as_str()).collect();
assert_eq!(applied, ["bool.and.true-left", "bool.not.double"]);
let output = report.output.as_ref().unwrap();
assert_eq!(output.nodes().len(), 1); // the normal form is p3

assert_eq!(replay(&input, &report).status, ReplayStatus::Verified);

let check = check_equivalence(&input, output, "demo", ConformanceOptions::default());
assert_eq!(check.status, ConformanceStatus::Equivalent);
```

More examples, including budgets, counterexamples and strict report reading, are
in [docs/examples.md](docs/examples.md).

## Reference

| Item | What it does |
| --- | --- |
| `rewrite`, `rewrite_with_context` | Rewrite a closed-trace or origin-history formula document to a fixed point and return a `RewriteReport`. The contextual form also binds the formula to a signal catalog. |
| `replay`, `replay_with_context` | Re-run a report's request and return a `ReplayReport` that is `Verified` or `Mismatch`. |
| `catalog`, `past_catalog`, `infinite_catalog` | The three immutable rule catalogs, each with a version and a SHA-256 digest. |
| `check_equivalence`, `check_equivalence_with_context` | Compare two closed-trace formulas on every trace within the horizon. |
| `check_past_equivalence` | Compare two origin-history formulas at one history and anchor. |
| `rewrite_infinite`, `replay_infinite` | Rewrite and replay infinite-trace formula graphs. |
| `check_infinite_rewrite` | Compare an infinite rewrite on one lasso trace (feature `infinite-trace`). |
| `report::{read, read_with_context}` | Read a canonical report from bytes and replay it before returning it. |

| Catalog | Rules | Enabled | Profile |
| --- | ---: | ---: | --- |
| `catalog()` | 40 | 38 | `mltl.closed-trace/v1` |
| `past_catalog()` | 26 | 26 | `mltl.origin-complete-history/v1` |
| `infinite_catalog()` | 42 | 40 | `mltl.infinite-trace/v1` |

| Feature | Default | Adds |
| --- | ---: | --- |
| none | yes | Rewriting, replay, equivalence checks, and infinite-trace rewriting and replay. |
| `infinite-trace` | no | `check_infinite_rewrite`, which evaluates both graphs with tl-mltl on a lasso trace. |

See [docs/api-reference.md](docs/api-reference.md) and [docs/rules.md](docs/rules.md).

## The TL stack

tl-rewrite works on tl-syntax formulas and uses the tl-mltl evaluator for its equivalence checks.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/tl-stack-dark.svg">
  <img alt="The TL crates. tl-syntax defines the formula graph, profiles and signal catalog, and tl-parse, tl-oracle, tl-mltl, tl-rewrite and quire-mltl all depend on it. tl-rewrite depends on tl-mltl. quire-mltl depends on tl-mltl and quire-observation, and is the single bridge to Quire. Dashed lines are development-only: tl-mltl and tl-rewrite test against tl-oracle, and tl-rewrite tests against tl-parse." src="docs/images/tl-stack-light.svg">
</picture>

## Building and testing

The full local gate is `make guarded-ci`. It runs the checks below and compares
the gates that completed with the declared list.

```sh
make guarded-ci
```

Individual targets:

```sh
make test              # cargo test --all-targets --all-features
make conformance       # replay every catalog rule through the engine and tl-mltl
make counterexamples   # replay the counterexample corpus
make normalization     # sweep a generated formula family
make spec              # validate the spec and the rule derivations with quire
```

The conformance, counterexample and
normalization targets run the programs in `examples/` and print one JSON row per
case.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) and [CLA.md](CLA.md). The change history is in
[CHANGELOG.md](CHANGELOG.md).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or the
[MIT license](LICENSE-MIT) at your option.
