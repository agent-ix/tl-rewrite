mod common;

use common::{document, proposition};
use tl_rewrite::{
    catalog, replay, rewrite, ReplayStatus, RewriteBudgets, RewriteOptions, RewriteStatus,
};
use tl_syntax::{Node, NodeId, NodeKind, SemanticProfile, SourceSpan};

fn traced_input() -> tl_syntax::FormulaDocument {
    document(
        SemanticProfile::ClosedTraceV1,
        vec![
            proposition(1),
            Node::with_span(
                NodeKind::Or {
                    left: NodeId(0),
                    right: NodeId(0),
                },
                SourceSpan::new(4, 9).unwrap(),
            ),
        ],
    )
}

// Trace: TC-009, FR-003-AC-1
#[test]
fn every_change_has_one_complete_ordered_step() {
    let report = rewrite(
        &traced_input(),
        "trace",
        RewriteOptions::default(),
        "source",
    );
    assert_eq!(report.rule_applications, report.steps.len() as u64);
    for (sequence, step) in report.steps.iter().enumerate() {
        assert_eq!(step.sequence, sequence as u64);
        let definition = catalog()
            .rules
            .into_iter()
            .find(|rule| rule.id == step.rule_id)
            .unwrap();
        assert_eq!(step.rule_revision, definition.revision);
        assert_eq!(step.before_sha256.len(), 64);
        assert_eq!(step.after_sha256.len(), 64);
        assert_eq!(step.intermediate_sha256.len(), 64);
    }
    assert_eq!(report.steps[0].source_span.unwrap().start(), 4);
}

#[test]
fn discarded_subtree_rewrites_are_absent_from_the_trace_and_budget() {
    let input = document(
        SemanticProfile::ClosedTraceV1,
        vec![
            Node::new(NodeKind::True),
            proposition(7),
            Node::new(NodeKind::Or {
                left: NodeId(0),
                right: NodeId(1),
            }),
            Node::new(NodeKind::False),
            Node::new(NodeKind::And {
                left: NodeId(2),
                right: NodeId(3),
            }),
        ],
    );
    let report = rewrite(
        &input,
        "discarded",
        RewriteOptions {
            budgets: RewriteBudgets {
                max_rule_applications: 1,
                ..RewriteBudgets::default()
            },
            ..RewriteOptions::default()
        },
        "source",
    );
    assert_eq!(report.status, RewriteStatus::Normalized);
    assert_eq!(report.rule_applications, 1);
    assert_eq!(report.steps.len(), 1);
    assert_eq!(report.steps[0].rule_id, "bool.and.false-right");
}

#[test]
fn structural_interning_does_not_resurrect_a_discarded_step() {
    let input = document(
        SemanticProfile::ClosedTraceV1,
        vec![
            Node::new(NodeKind::True),
            proposition(9),
            Node::new(NodeKind::Or {
                left: NodeId(0),
                right: NodeId(1),
            }),
            Node::new(NodeKind::Or {
                left: NodeId(0),
                right: NodeId(2),
            }),
        ],
    );
    let report = rewrite(
        &input,
        "interned-discard",
        RewriteOptions {
            budgets: RewriteBudgets {
                max_rule_applications: 1,
                ..RewriteBudgets::default()
            },
            ..RewriteOptions::default()
        },
        "source",
    );
    assert_eq!(report.status, RewriteStatus::Normalized);
    assert_eq!(report.steps.len(), 1);
    assert_eq!(report.steps[0].source_node, 3);
    assert_eq!(report.steps[0].rule_id, "bool.or.true-left");
}

// Trace: TC-010, FR-003-AC-2, StR-001-VC-2
#[test]
fn exact_replay_verifies() {
    let input = traced_input();
    let report = rewrite(&input, "trace", RewriteOptions::default(), "source");
    let replay = replay(&input, &report);
    assert_eq!(replay.status, ReplayStatus::Verified);
    assert_eq!(replay.expected_report_sha256, replay.observed_report_sha256);
}

// Trace: TC-011, FR-003-AC-2
#[test]
fn replay_rejects_changed_input_catalog_options_and_intermediate() {
    let input = traced_input();
    let report = rewrite(&input, "trace", RewriteOptions::default(), "source");

    let changed_input = document(SemanticProfile::ClosedTraceV1, vec![proposition(2)]);
    assert_eq!(
        replay(&changed_input, &report).status,
        ReplayStatus::Mismatch
    );

    let mut changed = report.clone();
    changed.catalog_sha256 = "0".repeat(64);
    assert_eq!(replay(&input, &changed).status, ReplayStatus::Mismatch);
    changed = report.clone();
    changed.options.budgets.max_work_units += 1;
    assert_eq!(replay(&input, &changed).status, ReplayStatus::Mismatch);
    changed = report;
    changed.steps[0].intermediate_sha256 = "f".repeat(64);
    assert_eq!(replay(&input, &changed).status, ReplayStatus::Mismatch);
}

// Trace: TC-012, FR-003-AC-3
#[test]
fn partial_attempt_has_diagnostics_but_no_success_output() {
    let input = traced_input();
    let report = rewrite(
        &input,
        "partial",
        RewriteOptions {
            budgets: RewriteBudgets {
                max_rule_applications: 0,
                ..RewriteBudgets::default()
            },
            ..RewriteOptions::default()
        },
        "source",
    );
    assert_eq!(report.status, RewriteStatus::BudgetExhausted);
    assert!(report.partial_sha256.is_some());
    assert!(report.detail.is_some());
    assert!(report.output.is_none());
    assert!(report.output_sha256.is_none());
}
