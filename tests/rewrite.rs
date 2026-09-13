mod common;

use common::{document, proposition};
use tl_parse::{parse, ParseLimits};
use tl_rewrite::{
    rewrite, BudgetKind, RewriteBudgets, RewriteOptions, RewriteStatus, RewriteStrategy,
};
use tl_syntax::{Node, NodeId, NodeKind, SemanticProfile, SourceSpan};

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

fn span(start: u32, end: u32) -> SourceSpan {
    SourceSpan::new(start, end).expect("ordered test span")
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

// Trace: TC-040, FR-002-AC-4
#[test]
fn span_distinct_parser_shaped_inputs_intern_and_identify_semantically() {
    let formula = |right_span, implication| {
        let root_kind = if implication {
            NodeKind::Implies {
                left: NodeId(0),
                right: NodeId(1),
            }
        } else {
            NodeKind::Or {
                left: NodeId(0),
                right: NodeId(1),
            }
        };
        document(
            SemanticProfile::ClosedTraceV1,
            vec![
                Node::with_span(proposition(1).kind, span(0, 2)),
                Node::with_span(proposition(1).kind, right_span),
                Node::with_span(root_kind, span(0, right_span.end())),
            ],
        )
    };
    for (implication, expected_rule) in [
        (false, "bool.or.idempotent"),
        (true, "bool.implies.reflexive"),
    ] {
        let compact = formula(span(3, 5), implication);
        let spaced = formula(span(5, 7), implication);
        let first = rewrite(
            &compact,
            "span-insensitive",
            RewriteOptions::default(),
            "source",
        );
        let second = rewrite(
            &spaced,
            "span-insensitive",
            RewriteOptions::default(),
            "source",
        );

        assert_eq!(first.input_sha256, second.input_sha256);
        assert_eq!(first.request_sha256, second.request_sha256);
        assert_eq!(first.output_sha256, second.output_sha256);
        assert_ne!(first.steps[0].source_span, second.steps[0].source_span);
        assert_eq!(
            first
                .steps
                .iter()
                .map(|step| (
                    &step.rule_id,
                    &step.before_sha256,
                    &step.after_sha256,
                    &step.intermediate_sha256
                ))
                .collect::<Vec<_>>(),
            second
                .steps
                .iter()
                .map(|step| (
                    &step.rule_id,
                    &step.before_sha256,
                    &step.after_sha256,
                    &step.intermediate_sha256
                ))
                .collect::<Vec<_>>()
        );
        assert_eq!(first.status, RewriteStatus::Normalized);
        assert_eq!(second.status, RewriteStatus::Normalized);
        assert!(first.steps.iter().any(|step| step.rule_id == expected_rule));
    }
}

// Trace: TC-040, FR-002-AC-4
#[test]
fn span_distinct_inputs_keep_budget_partial_identity_semantic() {
    let formula = |right_span| {
        document(
            SemanticProfile::ClosedTraceV1,
            vec![
                Node::with_span(proposition(1).kind, span(0, 2)),
                Node::with_span(proposition(1).kind, right_span),
                Node::with_span(
                    NodeKind::Or {
                        left: NodeId(0),
                        right: NodeId(1),
                    },
                    span(0, right_span.end()),
                ),
            ],
        )
    };
    let options = RewriteOptions {
        budgets: RewriteBudgets {
            max_rule_applications: 0,
            ..RewriteBudgets::default()
        },
        ..RewriteOptions::default()
    };
    let compact = rewrite(&formula(span(3, 5)), "partial", options, "source");
    let spaced = rewrite(&formula(span(5, 7)), "partial", options, "source");

    assert_eq!(compact.status, RewriteStatus::BudgetExhausted);
    assert_eq!(spaced.status, RewriteStatus::BudgetExhausted);
    assert_eq!(compact.partial_sha256, spaced.partial_sha256);
}

// Trace: TC-040, FR-002-AC-4
#[test]
fn parser_to_rewriter_seam_preserves_semantic_identity() {
    let parsed = |source| {
        parse(
            source,
            SemanticProfile::ClosedTraceV1,
            ParseLimits::default(),
        )
        .document
        .expect("parser accepts fixture")
    };
    let compact = parsed("p1|p1");
    let spaced = parsed("( p1 ) | p1");
    let first = rewrite(&compact, "parser-seam", RewriteOptions::default(), "source");
    let second = rewrite(&spaced, "parser-seam", RewriteOptions::default(), "source");

    assert_eq!(first.input_sha256, second.input_sha256);
    assert_eq!(first.request_sha256, second.request_sha256);
    assert_eq!(first.output_sha256, second.output_sha256);
    assert!(first
        .steps
        .iter()
        .any(|step| step.rule_id == "bool.or.idempotent"));
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

// Trace: TC-021, FR-002-AC-2, NFR-001-AC-3
#[test]
fn retained_source_selection_is_charged_to_the_work_budget() {
    let input = document(SemanticProfile::ClosedTraceV1, vec![proposition(0)]);
    let report = rewrite(
        &input,
        "retained-source-work",
        RewriteOptions {
            budgets: RewriteBudgets {
                max_work_units: 2,
                ..RewriteBudgets::default()
            },
            ..RewriteOptions::default()
        },
        "source",
    );
    assert_eq!(report.status, RewriteStatus::BudgetExhausted);
    assert_eq!(report.exhausted_budget, Some(BudgetKind::WorkUnits));
    assert!(report.output.is_none());

    let mut nodes = vec![proposition(0)];
    for index in 0..20 {
        nodes.push(Node::new(NodeKind::And {
            left: NodeId(index),
            right: NodeId(index),
        }));
    }
    let shared = document(SemanticProfile::ClosedTraceV1, nodes);
    let report = rewrite(
        &shared,
        "retained-source-dedup",
        RewriteOptions::default(),
        "source",
    );
    assert_eq!(report.status, RewriteStatus::Normalized);
    assert_eq!(report.work_units, 46);
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

// Trace: TC-007, FR-002-AC-2, NFR-001-AC-2
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

// Trace: TC-008, FR-002-AC-3
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
