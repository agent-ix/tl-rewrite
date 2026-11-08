#![no_main]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use tl_oracle::{
    evaluate_closed_trace_v1, evaluate_origin_complete, Formula as OracleFormula,
    Interval as OracleInterval, Limits,
};
use tl_rewrite::{rewrite, RewriteBudgets, RewriteOptions, RewriteStatus};
use tl_syntax::{
    FormulaDocument, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile,
};

const MAX_NODES: usize = 24;
const PROPOSITIONS: u32 = 3;
const MAX_TREE: usize = 4096;
const MAX_TRACE: usize = 6;

type Cells = Vec<BTreeMap<PropositionId, bool>>;

fn interval(u: &mut Unstructured<'_>) -> Option<Interval> {
    let start = u.int_in_range(0..=3_u32).ok()?;
    let width = u.int_in_range(0..=3_u32).ok()?;
    Interval::new(start, start + width).ok()
}

// Structure-aware generation: every node references only earlier nodes, so
// the document is a valid DAG by construction and no input is wasted on
// parse failures. `past` selects the origin-complete operator family.
fn formula(u: &mut Unstructured<'_>, past: bool) -> Option<FormulaDocument> {
    let count = u.int_in_range(1..=MAX_NODES).ok()?;
    let mut nodes: Vec<Node> = Vec::with_capacity(count);
    for index in 0..count {
        // Bias operands toward recent nodes so formulas nest deeply.
        let operand = |u: &mut Unstructured<'_>| -> Option<NodeId> {
            let back = u.int_in_range(0..=index.saturating_sub(1).min(5)).ok()?;
            Some(NodeId(u32::try_from(index - 1 - back).ok()?))
        };
        let kind = if index == 0 {
            match u.int_in_range(0..=3_u8).ok()? {
                0 => NodeKind::True,
                1 => NodeKind::False,
                _ => NodeKind::Proposition {
                    proposition: PropositionId(u.int_in_range(0..=PROPOSITIONS - 1).ok()?),
                },
            }
        } else {
            match u.int_in_range(0..=if past { 12_u8 } else { 11 }).ok()? {
                0 => NodeKind::True,
                1 => NodeKind::False,
                2 => NodeKind::Proposition {
                    proposition: PropositionId(u.int_in_range(0..=PROPOSITIONS - 1).ok()?),
                },
                3 => NodeKind::Not {
                    operand: operand(u)?,
                },
                4 => NodeKind::And {
                    left: operand(u)?,
                    right: operand(u)?,
                },
                5 => NodeKind::Or {
                    left: operand(u)?,
                    right: operand(u)?,
                },
                6 => NodeKind::Implies {
                    left: operand(u)?,
                    right: operand(u)?,
                },
                7 => NodeKind::Equivalent {
                    left: operand(u)?,
                    right: operand(u)?,
                },
                8 if !past => NodeKind::Future {
                    interval: interval(u)?,
                    operand: operand(u)?,
                },
                9 if !past => NodeKind::Globally {
                    interval: interval(u)?,
                    operand: operand(u)?,
                },
                10 if !past => NodeKind::Until {
                    interval: interval(u)?,
                    left: operand(u)?,
                    right: operand(u)?,
                },
                11 if !past => NodeKind::Release {
                    interval: interval(u)?,
                    left: operand(u)?,
                    right: operand(u)?,
                },
                8 => NodeKind::Once {
                    interval: interval(u)?,
                    operand: operand(u)?,
                },
                9 => NodeKind::Historically {
                    interval: interval(u)?,
                    operand: operand(u)?,
                },
                10 => NodeKind::Since {
                    interval: interval(u)?,
                    left: operand(u)?,
                    right: operand(u)?,
                },
                11 => NodeKind::Triggered {
                    interval: interval(u)?,
                    left: operand(u)?,
                    right: operand(u)?,
                },
                _ => NodeKind::StrongPrevious {
                    operand: operand(u)?,
                },
            }
        };
        nodes.push(Node::new(kind));
    }
    let root = NodeId(u32::try_from(nodes.len() - 1).ok()?);
    let profile = if past {
        SemanticProfile::OriginCompleteHistoryV1
    } else {
        SemanticProfile::ClosedTraceV1
    };
    FormulaDocument::new_v2(profile, root, nodes).ok()
}

fn trace(u: &mut Unstructured<'_>) -> Option<Cells> {
    let len = u.int_in_range(1..=MAX_TRACE).ok()?;
    (0..len)
        .map(|_| {
            (0..PROPOSITIONS)
                .map(|id| Some((PropositionId(id), u.arbitrary::<bool>().ok()?)))
                .collect()
        })
        .collect()
}

// The oracle consumes owned trees, so a shared-operand DAG expands to its
// unshared size. Bound that expansion before converting.
fn tree_size(document: &FormulaDocument) -> usize {
    let mut sizes: Vec<usize> = Vec::with_capacity(document.nodes().len());
    for node in document.nodes() {
        let at = |id: NodeId| sizes[usize::try_from(id.0).unwrap()];
        let size = match node.kind {
            NodeKind::True | NodeKind::False | NodeKind::Proposition { .. } => 1,
            NodeKind::Not { operand }
            | NodeKind::Future { operand, .. }
            | NodeKind::Globally { operand, .. }
            | NodeKind::Once { operand, .. }
            | NodeKind::Historically { operand, .. }
            | NodeKind::StrongPrevious { operand } => 1 + at(operand),
            NodeKind::And { left, right }
            | NodeKind::Or { left, right }
            | NodeKind::Implies { left, right }
            | NodeKind::Equivalent { left, right }
            | NodeKind::Until { left, right, .. }
            | NodeKind::Release { left, right, .. }
            | NodeKind::Since { left, right, .. }
            | NodeKind::Triggered { left, right, .. } => {
                1usize.saturating_add(at(left)).saturating_add(at(right))
            }
        };
        sizes.push(size.min(MAX_TREE + 1));
    }
    sizes[usize::try_from(document.root().0).unwrap()]
}

// Adapts public syntax structure into oracle-owned trees, from the root only so
// unreferenced nodes are never expanded; no evaluator logic or production
// semantic function is shared.
fn oracle_tree(document: &FormulaDocument, id: NodeId) -> OracleFormula {
    use OracleFormula as F;
    let b = |id: NodeId| Box::new(oracle_tree(document, id));
    let span = |value: Interval| OracleInterval::Closed {
        start: usize::try_from(value.start()).unwrap(),
        end: usize::try_from(value.end()).unwrap(),
    };
    match document.nodes()[usize::try_from(id.0).unwrap()].kind {
        NodeKind::False => F::False,
        NodeKind::True => F::True,
        NodeKind::Proposition { proposition } => F::Atom(proposition),
        NodeKind::Not { operand } => F::Not(b(operand)),
        NodeKind::And { left, right } => F::And(b(left), b(right)),
        NodeKind::Or { left, right } => F::Or(b(left), b(right)),
        NodeKind::Implies { left, right } => F::Implies(b(left), b(right)),
        NodeKind::Equivalent { left, right } => F::Equivalent(b(left), b(right)),
        NodeKind::Future { interval, operand } => F::Future(span(interval), b(operand)),
        NodeKind::Globally { interval, operand } => F::Globally(span(interval), b(operand)),
        NodeKind::Until {
            interval,
            left,
            right,
        } => F::Until(span(interval), b(left), b(right)),
        NodeKind::Release {
            interval,
            left,
            right,
        } => F::Release(span(interval), b(left), b(right)),
        NodeKind::Once { interval, operand } => F::Once(span(interval), b(operand)),
        NodeKind::Historically { interval, operand } => F::Historically(span(interval), b(operand)),
        NodeKind::StrongPrevious { operand } => F::StrongPrevious(b(operand)),
        NodeKind::Since {
            interval,
            left,
            right,
        } => F::Since(span(interval), b(left), b(right)),
        NodeKind::Triggered {
            interval,
            left,
            right,
        } => F::Triggered(span(interval), b(left), b(right)),
    }
}

// Trace: TC-066, TC-070, FR-004-AC-1, FR-020-AC-1.
fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let Ok(past) = u.arbitrary::<bool>() else {
        return;
    };
    let Some(input) = formula(&mut u, past) else {
        return;
    };
    let Some(cells) = trace(&mut u) else { return };
    if tree_size(&input) > MAX_TREE {
        return;
    }
    let options = RewriteOptions {
        budgets: RewriteBudgets {
            max_iterations: 8,
            max_nodes: 256,
            max_rule_applications: 256,
            max_work_units: 8192,
        },
        ..RewriteOptions::default()
    };
    let report = rewrite(&input, "fuzz-closed", options, "fuzz/v1");
    // A refused or non-conclusive rewrite must carry no output, so it can
    // never yield a different verdict.
    let Some(output) = report.output.as_ref() else {
        assert!(!matches!(
            report.status,
            RewriteStatus::Unchanged | RewriteStatus::Normalized
        ));
        return;
    };
    let evaluate = if past {
        evaluate_origin_complete
    } else {
        evaluate_closed_trace_v1
    };
    if tree_size(output) > MAX_TREE {
        return;
    }
    let before = oracle_tree(&input, input.root());
    let after = oracle_tree(output, output.root());
    let limits = Limits::default();
    for position in 0..cells.len() {
        let Ok(expected) = evaluate(&before, &cells, position, limits) else {
            continue;
        };
        let actual = evaluate(&after, &cells, position, limits).unwrap_or_else(|error| {
            panic!("oracle refused rewritten output at {position}: {error:?}")
        });
        assert_eq!(
            actual, expected,
            "rewrite changed verdict: position={position} cells={cells:?} input={input:?} output={output:?}"
        );
    }
});
