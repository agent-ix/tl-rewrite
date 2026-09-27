use std::collections::BTreeMap;

use tl_oracle::{
    evaluate_closed_trace_v1, evaluate_origin_complete, Formula as OracleFormula,
    Interval as OracleInterval, Limits,
};
use tl_rewrite::{
    catalog, check_equivalence, past_catalog, rewrite, ConformanceOptions, ConformanceStatus,
    RewriteOptions, RewriteStatus, RuleDisposition,
};
use tl_syntax::{
    FormulaDocument, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile,
};

type ReferenceFn = fn(
    &OracleFormula,
    &[BTreeMap<PropositionId, bool>],
    usize,
    Limits,
) -> Result<bool, tl_oracle::OracleError>;

fn fixture(family: &str, profile: SemanticProfile) -> FormulaDocument {
    let p = NodeId(0);
    let q = NodeId(1);
    let yes = NodeId(2);
    let no = NodeId(3);
    let not_p = NodeId(4);
    let not_q = NodeId(5);
    let mut nodes = vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(1),
        }),
        Node::new(NodeKind::True),
        Node::new(NodeKind::False),
        Node::new(NodeKind::Not { operand: p }),
        Node::new(NodeKind::Not { operand: q }),
    ];
    let zero = Interval::new(0, 0).unwrap();
    let root = match family {
        "bool.not" => NodeKind::Not { operand: no },
        "bool.and" => NodeKind::And { left: no, right: p },
        "bool.or" => NodeKind::Or {
            left: yes,
            right: p,
        },
        "bool.implies" => NodeKind::Implies { left: no, right: p },
        "bool.equivalent" => NodeKind::Equivalent { left: p, right: p },
        "temporal.future" => NodeKind::Future {
            interval: zero,
            operand: p,
        },
        "temporal.globally" => NodeKind::Globally {
            interval: zero,
            operand: p,
        },
        "temporal.until" => NodeKind::Until {
            interval: zero,
            left: p,
            right: q,
        },
        "temporal.release" => NodeKind::Release {
            interval: zero,
            left: p,
            right: q,
        },
        "neg.future" | "neg.globally" | "neg.until" | "neg.release" => {
            let interval = Interval::new(1, 2).unwrap();
            let inner = match family {
                "neg.future" => NodeKind::Future {
                    interval,
                    operand: p,
                },
                "neg.globally" => NodeKind::Globally {
                    interval,
                    operand: p,
                },
                "neg.until" => NodeKind::Until {
                    interval,
                    left: p,
                    right: q,
                },
                "neg.release" => NodeKind::Release {
                    interval,
                    left: p,
                    right: q,
                },
                _ => unreachable!(),
            };
            nodes.push(Node::new(inner));
            NodeKind::Not { operand: NodeId(6) }
        }
        "past.once" => NodeKind::Once {
            interval: Interval::new(1, 1).unwrap(),
            operand: p,
        },
        "past.triggered" => {
            nodes.push(Node::new(NodeKind::Since {
                interval: Interval::new(1, 2).unwrap(),
                left: not_p,
                right: not_q,
            }));
            NodeKind::Not { operand: NodeId(6) }
        }
        other => panic!("uncovered catalog family: {other}"),
    };
    let root_id = NodeId(u32::try_from(nodes.len()).unwrap());
    nodes.push(Node::new(root));
    FormulaDocument::new_v2(profile, root_id, nodes).unwrap()
}

// Adapts public syntax structure into oracle-owned trees; no evaluator logic
// or production semantic function is shared.
fn oracle_tree(document: &FormulaDocument) -> OracleFormula {
    use OracleFormula as F;
    let mut trees: Vec<OracleFormula> = Vec::with_capacity(document.nodes().len());
    for node in document.nodes() {
        let child = |id: NodeId| trees[usize::try_from(id.0).unwrap()].clone();
        let interval = |value: Interval| OracleInterval::Closed {
            start: usize::try_from(value.start()).unwrap(),
            end: usize::try_from(value.end()).unwrap(),
        };
        let tree = match node.kind {
            NodeKind::False => F::False,
            NodeKind::True => F::True,
            NodeKind::Proposition { proposition } => F::Atom(proposition),
            NodeKind::Not { operand } => F::Not(Box::new(child(operand))),
            NodeKind::And { left, right } => F::And(Box::new(child(left)), Box::new(child(right))),
            NodeKind::Or { left, right } => F::Or(Box::new(child(left)), Box::new(child(right))),
            NodeKind::Implies { left, right } => {
                F::Implies(Box::new(child(left)), Box::new(child(right)))
            }
            NodeKind::Equivalent { left, right } => {
                F::Equivalent(Box::new(child(left)), Box::new(child(right)))
            }
            NodeKind::Future {
                interval: i,
                operand,
            } => F::Future(interval(i), Box::new(child(operand))),
            NodeKind::Globally {
                interval: i,
                operand,
            } => F::Globally(interval(i), Box::new(child(operand))),
            NodeKind::Until {
                interval: i,
                left,
                right,
            } => F::Until(interval(i), Box::new(child(left)), Box::new(child(right))),
            NodeKind::Release {
                interval: i,
                left,
                right,
            } => F::Release(interval(i), Box::new(child(left)), Box::new(child(right))),
            NodeKind::Once {
                interval: i,
                operand,
            } => F::Once(interval(i), Box::new(child(operand))),
            NodeKind::Historically {
                interval: i,
                operand,
            } => F::Historically(interval(i), Box::new(child(operand))),
            NodeKind::StrongPrevious { operand } => F::StrongPrevious(Box::new(child(operand))),
            NodeKind::Since {
                interval: i,
                left,
                right,
            } => F::Since(interval(i), Box::new(child(left)), Box::new(child(right))),
            NodeKind::Triggered {
                interval: i,
                left,
                right,
            } => F::Triggered(interval(i), Box::new(child(left)), Box::new(child(right))),
        };
        trees.push(tree);
    }
    trees[usize::try_from(document.root().0).unwrap()].clone()
}

fn words() -> Vec<Vec<BTreeMap<PropositionId, bool>>> {
    (0_u16..64)
        .map(|bits| {
            (0..3)
                .map(|position| {
                    [0_u32, 1]
                        .into_iter()
                        .map(|proposition| {
                            let bit = position * 2 + proposition;
                            (PropositionId(proposition), bits & (1 << bit) != 0)
                        })
                        .collect()
                })
                .collect()
        })
        .collect()
}

// Trace: TC-066, TC-070; FR-004-AC-1, FR-020-AC-1.
#[test]
fn every_finite_and_past_catalog_family_agrees_with_independent_oracle() {
    let finite_families = [
        "bool.not",
        "bool.and",
        "bool.or",
        "bool.implies",
        "bool.equivalent",
        "temporal.future",
        "temporal.globally",
        "temporal.until",
        "temporal.release",
        "neg.future",
        "neg.globally",
        "neg.until",
        "neg.release",
    ];
    let past_families = [
        "bool.not",
        "bool.and",
        "bool.or",
        "bool.implies",
        "bool.equivalent",
        "past.once",
        "past.triggered",
    ];
    let mut compared = 0;
    for (profile, families, catalog) in [
        (
            SemanticProfile::ClosedTraceV1,
            &finite_families[..],
            catalog(),
        ),
        (
            SemanticProfile::OriginCompleteHistoryV1,
            &past_families[..],
            past_catalog(),
        ),
    ] {
        for rule in catalog
            .rules
            .iter()
            .filter(|rule| rule.disposition == RuleDisposition::Enabled)
        {
            assert!(
                families
                    .iter()
                    .any(|family| rule.id.starts_with(&format!("{family}."))),
                "uncovered {profile:?} rule family: {}",
                rule.id
            );
        }
        for family in families {
            let input = fixture(family, profile);
            let report = rewrite(&input, "oracle-family", RewriteOptions::default(), "test");
            assert!(
                matches!(
                    report.status,
                    RewriteStatus::Unchanged | RewriteStatus::Normalized
                ) && report.output.is_some(),
                "{family}: {:?}",
                report.status
            );
            assert!(
                report
                    .steps
                    .iter()
                    .any(|step| step.rule_id.starts_with(family)),
                "{family} never executed"
            );
            let before = oracle_tree(&input);
            let after = oracle_tree(report.output.as_ref().unwrap());
            if profile == SemanticProfile::ClosedTraceV1 && *family == "temporal.future" {
                // Runtime comparison is differential evidence; the oracle
                // enumeration below independently qualifies the rule family.
                let diagnostic = check_equivalence(
                    &input,
                    report.output.as_ref().unwrap(),
                    "oracle-family",
                    ConformanceOptions::default(),
                );
                assert_eq!(diagnostic.status, ConformanceStatus::Equivalent);
            }
            for word in words() {
                for position in 0..word.len() {
                    let evaluate = match profile {
                        SemanticProfile::ClosedTraceV1 => evaluate_closed_trace_v1,
                        SemanticProfile::OriginCompleteHistoryV1 => evaluate_origin_complete,
                        other => panic!("uncovered qualification profile: {other:?}"),
                    };
                    let original = evaluate(&before, &word, position, Limits::default()).unwrap();
                    let rewritten = evaluate(&after, &word, position, Limits::default()).unwrap();
                    assert_eq!(
                        original, rewritten,
                        "{profile:?} {family}, word={word:?}, position={position}"
                    );
                    compared += 1;
                }
            }
        }
    }
    assert_eq!(compared, 3_840);
}

// Trace: TC-070; FR-020-AC-1.
#[test]
fn wrong_bounded_and_origin_past_rules_yield_concrete_counterexamples() {
    let p = PropositionId(0);
    let q = PropositionId(1);
    let word = vec![
        BTreeMap::from([(p, true), (q, false)]),
        BTreeMap::from([(p, false), (q, false)]),
    ];
    let bounded_claim = OracleFormula::Future(
        OracleInterval::Closed { start: 0, end: 1 },
        Box::new(OracleFormula::Atom(p)),
    );
    let past_claim = OracleFormula::Once(
        OracleInterval::Closed { start: 1, end: 1 },
        Box::new(OracleFormula::Atom(p)),
    );
    let cases: [(OracleFormula, usize, ReferenceFn); 2] = [
        (bounded_claim, 0, evaluate_closed_trace_v1),
        (past_claim, 1, evaluate_origin_complete),
    ];
    for (claim, position, evaluate) in cases {
        let actual = evaluate(&claim, &word, position, Limits::default()).unwrap();
        let seeded_wrong =
            evaluate(&OracleFormula::False, &word, position, Limits::default()).unwrap();
        assert!(
            actual && !seeded_wrong,
            "word={word:?}, position={position}"
        );
    }
}
