mod common;

use common::{document, proposition};
use tl_rewrite::{
    rewrite, BudgetKind, RewriteBudgets, RewriteOptions, RewriteStatus, RewriteStrategy,
};
use tl_syntax::{Node, NodeId, NodeKind, SemanticProfile};

fn reducible() -> tl_syntax::FormulaDocument {
    document(
        SemanticProfile::ClosedTraceV1,
        vec![
            Node::new(NodeKind::True),
            proposition(3),
            Node::new(NodeKind::And {
                left: NodeId(0),
                right: NodeId(1),
            }),
        ],
    )
}

// Trace: TC-003, FR-001-AC-2
#[test]
fn primitive_and_nested_derivations_reach_expected_root() {
    let input = document(
        SemanticProfile::ClosedTraceV1,
        vec![
            Node::new(NodeKind::False),
            proposition(9),
            Node::new(NodeKind::Or {
                left: NodeId(0),
                right: NodeId(1),
            }),
            Node::new(NodeKind::True),
            Node::new(NodeKind::And {
                left: NodeId(3),
                right: NodeId(2),
            }),
        ],
    );
    let report = rewrite(&input, "nested", RewriteOptions::default(), "source");
    assert_eq!(report.status, RewriteStatus::Normalized);
    let output = report.output.unwrap();
    assert_eq!(
        output.nodes()[output.root().0 as usize].kind,
        proposition(9).kind
    );
    assert!(report
        .steps
        .iter()
        .any(|step| step.rule_id == "bool.or.false-left"));
    assert!(report
        .steps
        .iter()
        .any(|step| step.rule_id == "bool.and.true-left"));
}

// Trace: TC-005, FR-002-AC-1, NFR-001-AC-1, StR-002-VC-1
#[test]
fn identical_requests_are_byte_identical() {
    let input = reducible();
    let run = || rewrite(&input, "repeat", RewriteOptions::default(), "source");
    let first = run();
    assert_eq!(first, run());
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&run()).unwrap()
    );
}

// Trace: TC-006, FR-002-AC-2, NFR-001-AC-2
#[test]
fn iteration_application_and_work_budgets_fail_closed() {
    let input = reducible();
    let options = |budgets| RewriteOptions {
        strategy: RewriteStrategy::BottomUpFirstMatch,
        budgets,
    };
    let mut budgets = RewriteBudgets {
        max_iterations: 1,
        ..RewriteBudgets::default()
    };
    let report = rewrite(&input, "iteration", options(budgets), "source");
    assert_eq!(report.exhausted_budget, Some(BudgetKind::Iterations));
    assert!(report.output.is_none());

    budgets = RewriteBudgets {
        max_rule_applications: 0,
        ..RewriteBudgets::default()
    };
    let report = rewrite(&input, "application", options(budgets), "source");
    assert_eq!(report.exhausted_budget, Some(BudgetKind::RuleApplications));
    assert!(report.output.is_none());

    budgets = RewriteBudgets {
        max_work_units: 0,
        ..RewriteBudgets::default()
    };
    let report = rewrite(&input, "work", options(budgets), "source");
    assert_eq!(report.exhausted_budget, Some(BudgetKind::WorkUnits));
    assert!(report.output.is_none());
}

// Trace: TC-007, FR-002-AC-2, NFR-001-AC-2
#[test]
fn node_growth_limit_fails_before_emission() {
    let input = reducible();
    let options = RewriteOptions {
        strategy: RewriteStrategy::BottomUpFirstMatch,
        budgets: RewriteBudgets {
            max_nodes: 0,
            ..RewriteBudgets::default()
        },
    };
    let report = rewrite(&input, "nodes", options, "source");
    assert_eq!(report.status, RewriteStatus::BudgetExhausted);
    assert_eq!(report.exhausted_budget, Some(BudgetKind::Nodes));
    assert!(report.output.is_none());
}

#[test]
fn node_limit_applies_to_the_reachable_compacted_graph() {
    let input = document(
        SemanticProfile::ClosedTraceV1,
        vec![
            proposition(0),
            proposition(1),
            Node::new(NodeKind::And {
                left: NodeId(0),
                right: NodeId(1),
            }),
        ],
    );
    let report = rewrite(
        &input,
        "nondegenerate-node-limit",
        RewriteOptions {
            budgets: RewriteBudgets {
                max_nodes: 2,
                ..RewriteBudgets::default()
            },
            ..RewriteOptions::default()
        },
        "source",
    );
    assert_eq!(report.status, RewriteStatus::BudgetExhausted);
    assert_eq!(report.exhausted_budget, Some(BudgetKind::Nodes));
    assert!(report.output.is_none());
}

#[test]
fn output_prunes_unreachable_nodes_and_structurally_interns_duplicates() {
    let input = document(
        SemanticProfile::ClosedTraceV1,
        vec![proposition(4), proposition(4), Node::new(NodeKind::True)],
    );
    let report = rewrite(&input, "compact", RewriteOptions::default(), "source");
    assert_eq!(report.status, RewriteStatus::Normalized);
    assert!(report.steps.is_empty());
    let output = report.output.unwrap();
    assert_eq!(output.nodes(), &[Node::new(NodeKind::True)]);
    assert_eq!(output.root(), NodeId(0));
}

// Trace: TC-008, FR-002-AC-3
#[test]
fn output_preserves_profile_validity_and_fixed_point() {
    let input = reducible();
    let report = rewrite(&input, "fixed", RewriteOptions::default(), "source");
    let output = report.output.unwrap();
    assert_eq!(output.semantic_profile(), input.semantic_profile());
    output.validate().unwrap();
    let second = rewrite(&output, "fixed", RewriteOptions::default(), "source");
    assert_eq!(second.status, RewriteStatus::Unchanged);
    assert_eq!(second.output, Some(output));

    let online = document(SemanticProfile::OnlinePrefixV1, vec![proposition(0)]);
    let unsupported = rewrite(&online, "online", RewriteOptions::default(), "source");
    assert_eq!(unsupported.status, RewriteStatus::UnsupportedProfile);
    assert!(unsupported.output.is_none());
}

// Trace: TC-007, TC-008, FR-002-AC-2, FR-002-AC-3
#[test]
fn bounded_formula_family_is_terminating_idempotent_and_growth_limited() {
    let leaves = [NodeKind::False, NodeKind::True, proposition(0).kind];
    for (left_index, left) in leaves.into_iter().enumerate() {
        for (right_index, right) in leaves.into_iter().enumerate() {
            for operator in 0..4 {
                let root = match operator {
                    0 => NodeKind::And {
                        left: NodeId(0),
                        right: NodeId(1),
                    },
                    1 => NodeKind::Or {
                        left: NodeId(0),
                        right: NodeId(1),
                    },
                    2 => NodeKind::Implies {
                        left: NodeId(0),
                        right: NodeId(1),
                    },
                    _ => NodeKind::Equivalent {
                        left: NodeId(0),
                        right: NodeId(1),
                    },
                };
                let input = document(
                    SemanticProfile::ClosedTraceV1,
                    vec![Node::new(left), Node::new(right), Node::new(root)],
                );
                let id = format!("family-{left_index}-{right_index}-{operator}");
                let first = rewrite(&input, &id, RewriteOptions::default(), "source");
                assert!(matches!(
                    first.status,
                    RewriteStatus::Normalized | RewriteStatus::Unchanged
                ));
                let output = first.output.unwrap();
                output.validate().unwrap();
                assert!(output.nodes().len() <= 5);
                let second = rewrite(&output, id, RewriteOptions::default(), "source");
                assert_eq!(second.status, RewriteStatus::Unchanged);
                assert_eq!(second.output, Some(output));
            }
        }
    }
}
