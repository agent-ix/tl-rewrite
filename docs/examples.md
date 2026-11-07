# Examples

Every example here compiles and passes against the crate with the default features. The snippets assume these imports and helper:

```rust
use tl_rewrite::*;
use tl_syntax::{FormulaDocument, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

fn p(id: u32) -> Node {
    Node::new(NodeKind::Proposition { proposition: PropositionId(id) })
}
```

## Rewrite and replay

The README example rewrites `not (not (true and p3))` to `p3`, replays the report and compares the result with the input. A formula that is already a fixed point reports `Unchanged`:

```rust
let formula = FormulaDocument::new(SemanticProfile::ClosedTraceV1, NodeId(0), vec![p(0)]).unwrap();
let report = rewrite(&formula, "p", RewriteOptions::default(), "rev-1");
assert_eq!(report.status, RewriteStatus::Unchanged);
```

## A budget that is too small

A budget stops the attempt with a typed status and no output. This request allows no work at all:

```rust
let nodes = vec![
    Node::new(NodeKind::True),
    p(3),
    Node::new(NodeKind::And { left: NodeId(0), right: NodeId(1) }),
];
let input = FormulaDocument::new(SemanticProfile::ClosedTraceV1, NodeId(2), nodes).unwrap();

let mut options = RewriteOptions::default();
options.budgets.max_work_units = 0;
let report = rewrite(&input, "tight", options, "rev-1");
assert_eq!(report.status, RewriteStatus::BudgetExhausted);
assert_eq!(report.exhausted_budget, Some(BudgetKind::WorkUnits));
assert!(report.output.is_none());
```

## A counterexample

Replacing `p0` with `not p0` is not semantics-preserving. The comparison stops at the first trace where the two differ. The trace is a list of instants, and each instant lists the propositions that are true. Here the only instant has no true proposition:

```rust
let original = FormulaDocument::new(SemanticProfile::ClosedTraceV1, NodeId(0), vec![p(0)]).unwrap();
let wrong = FormulaDocument::new(
    SemanticProfile::ClosedTraceV1,
    NodeId(1),
    vec![p(0), Node::new(NodeKind::Not { operand: NodeId(0) })],
)
.unwrap();

let check = check_equivalence(&original, &wrong, "negated", ConformanceOptions::default());
assert_eq!(check.status, ConformanceStatus::Mismatch);
assert_eq!(check.counterexample, Some(vec![vec![]]));
assert_eq!(check.original_verdict, Some(tl_mltl::TruthValue::False));
assert_eq!(check.rewritten_verdict, Some(tl_mltl::TruthValue::True));
```

The example needs `tl-mltl` as a dependency to name `TruthValue`. The comparison itself does not need it in the caller.

## Replay detects a changed report

Removing a step from a report makes it differ from a fresh run of the same request:

```rust
let mut report = rewrite(&input, "ok", RewriteOptions::default(), "rev-1");
report.steps.pop();
assert_eq!(replay(&input, &report).status, ReplayStatus::Mismatch);
```

## Read a report from bytes

`report::read` checks the bytes, then replays the record against the input:

```rust
let bytes = serde_json::to_vec(&rewrite(&input, "ok", RewriteOptions::default(), "rev-1")).unwrap();
let report = tl_rewrite::report::read(&bytes, &input, RecordLimits::default()).unwrap();
assert_eq!(report.status, RewriteStatus::Normalized);

let mut padded = bytes.clone();
padded.push(b' ');
let error = tl_rewrite::report::read(&padded, &input, RecordLimits::default()).unwrap_err();
assert_eq!(error.code(), RecordReadErrorCode::NonCanonical);
```

The extra space is valid JSON but is not the canonical encoding, so the reader refuses it.

## The past catalog

A document with the origin-complete-history profile uses the past catalog. Past nodes need a version 2 document. `O[1,1] p0` folds to strong Previous:

```rust
let once = FormulaDocument::new_v2(
    SemanticProfile::OriginCompleteHistoryV1,
    NodeId(1),
    vec![
        p(0),
        Node::new(NodeKind::Once { interval: Interval::new(1, 1).unwrap(), operand: NodeId(0) }),
    ],
)
.unwrap();
let report = rewrite(&once, "once", RewriteOptions::default(), "rev-1");
assert_eq!(report.status, RewriteStatus::Normalized);
assert_eq!(report.steps[0].rule_id, "past.once.strong-previous");
```

The online-prefix profile has no rule set. `rewrite` returns `UnsupportedProfile` for it.

## Inspect a catalog

```rust
let rules = catalog().rules;
let enabled = rules.iter().filter(|r| r.disposition == RuleDisposition::Enabled).count();
assert_eq!((rules.len(), enabled), (40, 38));
assert_eq!(past_catalog().rules.len(), 26);
```

## Run the conformance programs

The programs in `examples/` take no formula input of their own. They read the corpora and print one JSON row per case.

```sh
cargo run --example rule_conformance -- --manifest corpus/rules/manifest.json
cargo run --example counterexample_evidence -- --manifest corpus/counterexamples/manifest.json
cargo run --release --example normalization_sweep
```

Each exits non-zero when a case fails. `rule_conformance` writes one row per catalog rule, 40 in all, with the two excluded rules reported as `unsupported`.
