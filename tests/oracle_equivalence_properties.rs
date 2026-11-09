//! Generated-formula property tests: every rewrite that yields an output
//! preserves the verdict of the independent `tl-oracle` evaluator on every
//! generated trace, and every rewrite that does not succeed leaves no output.
//! The oracle shares no evaluator logic with the production rewrite path.

use std::collections::BTreeMap;

use proptest::prelude::*;
use tl_oracle::{
    evaluate_closed_trace_v1, evaluate_origin_complete, Formula as OracleFormula,
    Interval as OracleInterval, Limits,
};
use tl_rewrite::{rewrite, RewriteOptions, RewriteStatus};
use tl_syntax::{
    FormulaDocument, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile,
};

type Word = Vec<BTreeMap<PropositionId, bool>>;

/// Recursive formula shape generated independently of any syntax document.
#[derive(Clone, Debug)]
enum Shape {
    True,
    False,
    Atom(u32),
    Not(Box<Shape>),
    Bin(u8, Box<Shape>, Box<Shape>),
    Unary(u8, u32, u32, Box<Shape>),
    Temporal(u8, u32, u32, Box<Shape>, Box<Shape>),
}

fn shape(past: bool) -> impl Strategy<Value = Shape> {
    let leaf = prop_oneof![
        Just(Shape::True),
        Just(Shape::False),
        (0_u32..2).prop_map(Shape::Atom),
    ];
    leaf.prop_recursive(3, 12, 2, move |inner| {
        let interval = (0_u32..3).prop_flat_map(|start| (Just(start), start..4));
        prop_oneof![
            inner.clone().prop_map(|s| Shape::Not(Box::new(s))),
            (0_u8..4, inner.clone(), inner.clone()).prop_map(|(k, l, r)| Shape::Bin(
                k,
                Box::new(l),
                Box::new(r)
            )),
            (0_u8..2, interval.clone(), inner.clone()).prop_map(
                move |(k, (a, b), o)| Shape::Unary(k + if past { 2 } else { 0 }, a, b, Box::new(o))
            ),
            (0_u8..2, interval, inner.clone(), inner).prop_map(move |(k, (a, b), l, r)| {
                Shape::Temporal(k + if past { 2 } else { 0 }, a, b, Box::new(l), Box::new(r))
            }),
        ]
    })
}

fn lower(shape: &Shape, nodes: &mut Vec<Node>) -> NodeId {
    let kind = match shape {
        Shape::True => NodeKind::True,
        Shape::False => NodeKind::False,
        Shape::Atom(p) => NodeKind::Proposition {
            proposition: PropositionId(*p),
        },
        Shape::Not(s) => NodeKind::Not {
            operand: lower(s, nodes),
        },
        Shape::Bin(k, l, r) => {
            let (left, right) = (lower(l, nodes), lower(r, nodes));
            match k {
                0 => NodeKind::And { left, right },
                1 => NodeKind::Or { left, right },
                2 => NodeKind::Implies { left, right },
                _ => NodeKind::Equivalent { left, right },
            }
        }
        Shape::Unary(k, a, b, o) => {
            let operand = lower(o, nodes);
            let interval = Interval::new(*a, *b).unwrap();
            match k {
                0 => NodeKind::Future { interval, operand },
                1 => NodeKind::Globally { interval, operand },
                2 => NodeKind::Once { interval, operand },
                _ => NodeKind::Historically { interval, operand },
            }
        }
        Shape::Temporal(k, a, b, l, r) => {
            let (left, right) = (lower(l, nodes), lower(r, nodes));
            let interval = Interval::new(*a, *b).unwrap();
            match k {
                0 => NodeKind::Until {
                    interval,
                    left,
                    right,
                },
                1 => NodeKind::Release {
                    interval,
                    left,
                    right,
                },
                2 => NodeKind::Since {
                    interval,
                    left,
                    right,
                },
                _ => NodeKind::Triggered {
                    interval,
                    left,
                    right,
                },
            }
        }
    };
    nodes.push(Node::new(kind));
    NodeId(u32::try_from(nodes.len() - 1).unwrap())
}

fn document(profile: SemanticProfile, shape: &Shape) -> FormulaDocument {
    let mut nodes = Vec::new();
    let root = lower(shape, &mut nodes);
    FormulaDocument::new_v2(profile, root, nodes).unwrap()
}

fn oracle_tree(shape: &Shape) -> OracleFormula {
    use OracleFormula as F;
    let b = |s: &Shape| Box::new(oracle_tree(s));
    let iv = |a: u32, z: u32| OracleInterval::Closed {
        start: usize::try_from(a).unwrap(),
        end: usize::try_from(z).unwrap(),
    };
    match shape {
        Shape::True => F::True,
        Shape::False => F::False,
        Shape::Atom(p) => F::Atom(PropositionId(*p)),
        Shape::Not(s) => F::Not(b(s)),
        Shape::Bin(k, l, r) => match k {
            0 => F::And(b(l), b(r)),
            1 => F::Or(b(l), b(r)),
            2 => F::Implies(b(l), b(r)),
            _ => F::Equivalent(b(l), b(r)),
        },
        Shape::Unary(k, a, z, o) => match k {
            0 => F::Future(iv(*a, *z), b(o)),
            1 => F::Globally(iv(*a, *z), b(o)),
            2 => F::Once(iv(*a, *z), b(o)),
            _ => F::Historically(iv(*a, *z), b(o)),
        },
        Shape::Temporal(k, a, z, l, r) => match k {
            0 => F::Until(iv(*a, *z), b(l), b(r)),
            1 => F::Release(iv(*a, *z), b(l), b(r)),
            2 => F::Since(iv(*a, *z), b(l), b(r)),
            _ => F::Triggered(iv(*a, *z), b(l), b(r)),
        },
    }
}

fn word() -> impl Strategy<Value = Word> {
    prop::collection::vec((any::<bool>(), any::<bool>()), 1..5).prop_map(|rows| {
        rows.into_iter()
            .map(|(p, q)| BTreeMap::from([(PropositionId(0), p), (PropositionId(1), q)]))
            .collect()
    })
}

fn document_oracle_tree(document: &FormulaDocument) -> OracleFormula {
    // Re-derive the oracle tree from the rewritten document structure.
    fn go(document: &FormulaDocument, id: NodeId) -> OracleFormula {
        use OracleFormula as F;
        let node = &document.nodes()[usize::try_from(id.0).unwrap()];
        let sub = |id: NodeId| Box::new(go(document, id));
        let iv = |i: Interval| OracleInterval::Closed {
            start: usize::try_from(i.start()).unwrap(),
            end: usize::try_from(i.end()).unwrap(),
        };
        match node.kind {
            NodeKind::False => F::False,
            NodeKind::True => F::True,
            NodeKind::Proposition { proposition } => F::Atom(proposition),
            NodeKind::Not { operand } => F::Not(sub(operand)),
            NodeKind::And { left, right } => F::And(sub(left), sub(right)),
            NodeKind::Or { left, right } => F::Or(sub(left), sub(right)),
            NodeKind::Implies { left, right } => F::Implies(sub(left), sub(right)),
            NodeKind::Equivalent { left, right } => F::Equivalent(sub(left), sub(right)),
            NodeKind::Future { interval, operand } => F::Future(iv(interval), sub(operand)),
            NodeKind::Globally { interval, operand } => F::Globally(iv(interval), sub(operand)),
            NodeKind::Until {
                interval,
                left,
                right,
            } => F::Until(iv(interval), sub(left), sub(right)),
            NodeKind::Release {
                interval,
                left,
                right,
            } => F::Release(iv(interval), sub(left), sub(right)),
            NodeKind::Once { interval, operand } => F::Once(iv(interval), sub(operand)),
            NodeKind::Historically { interval, operand } => {
                F::Historically(iv(interval), sub(operand))
            }
            NodeKind::StrongPrevious { operand } => F::StrongPrevious(sub(operand)),
            NodeKind::Since {
                interval,
                left,
                right,
            } => F::Since(iv(interval), sub(left), sub(right)),
            NodeKind::Triggered {
                interval,
                left,
                right,
            } => F::Triggered(iv(interval), sub(left), sub(right)),
        }
    }
    go(document, document.root())
}

fn check(profile: SemanticProfile, shape: &Shape, word: &Word) -> Result<(), TestCaseError> {
    let input = document(profile, shape);
    let report = rewrite(
        &input,
        "oracle-property",
        RewriteOptions::default(),
        "property",
    );
    match report.status {
        RewriteStatus::Unchanged | RewriteStatus::Normalized => {
            let output = report.output.as_ref().expect("success carries an output");
            let evaluate = if profile == SemanticProfile::ClosedTraceV1 {
                evaluate_closed_trace_v1
            } else {
                evaluate_origin_complete
            };
            let before = oracle_tree(shape);
            let after = document_oracle_tree(output);
            for position in 0..word.len() {
                let original = evaluate(&before, word, position, Limits::default());
                let rewritten = evaluate(&after, word, position, Limits::default());
                prop_assert_eq!(original.ok(), rewritten.ok(), "position {}", position);
            }
        }
        _ => {
            prop_assert!(report.output.is_none(), "refused rewrite kept an output");
            prop_assert!(report.steps.is_empty(), "refused rewrite kept steps");
        }
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    // Trace: FR-004-AC-1, FR-020-AC-1, FR-002-AC-2
    #[test]
    fn closed_trace_rewrites_preserve_the_independent_oracle_verdict(
        shape in shape(false),
        word in word(),
    ) {
        check(SemanticProfile::ClosedTraceV1, &shape, &word)?;
    }

    // Trace: FR-009-AC-3, FR-020-AC-1, FR-002-AC-2
    #[test]
    fn origin_past_rewrites_preserve_the_independent_oracle_verdict(
        shape in shape(true),
        word in word(),
    ) {
        check(SemanticProfile::OriginCompleteHistoryV1, &shape, &word)?;
    }
}
