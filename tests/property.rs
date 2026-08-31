mod common;

use common::{document, proposition};
use proptest::prelude::*;
use tl_rewrite::{
    check_equivalence, rewrite, ConformanceOptions, ConformanceStatus, RewriteOptions,
    RewriteStatus,
};
use tl_syntax::{Interval, Node, NodeId, NodeKind, SemanticProfile};

fn temporal_fixture(kind: u8, interval: Interval) -> (&'static str, tl_syntax::FormulaDocument) {
    match kind {
        0 | 1 => {
            let temporal = if kind == 0 {
                NodeKind::Until {
                    interval,
                    left: NodeId(0),
                    right: NodeId(1),
                }
            } else {
                NodeKind::Release {
                    interval,
                    left: NodeId(0),
                    right: NodeId(1),
                }
            };
            (
                if kind == 0 {
                    "neg.until.dual"
                } else {
                    "neg.release.dual"
                },
                document(
                    SemanticProfile::ClosedTraceV1,
                    vec![
                        proposition(0),
                        proposition(1),
                        Node::new(temporal),
                        Node::new(NodeKind::Not { operand: NodeId(2) }),
                    ],
                ),
            )
        }
        2 => (
            "temporal.until.true-left",
            document(
                SemanticProfile::ClosedTraceV1,
                vec![
                    Node::new(NodeKind::True),
                    proposition(0),
                    Node::new(NodeKind::Until {
                        interval,
                        left: NodeId(0),
                        right: NodeId(1),
                    }),
                ],
            ),
        ),
        _ => (
            "temporal.release.false-left",
            document(
                SemanticProfile::ClosedTraceV1,
                vec![
                    Node::new(NodeKind::False),
                    proposition(0),
                    Node::new(NodeKind::Release {
                        interval,
                        left: NodeId(0),
                        right: NodeId(1),
                    }),
                ],
            ),
        ),
    }
}

fn bounded_interval() -> impl Strategy<Value = Interval> {
    (1_u32..=3).prop_flat_map(|start| {
        (Just(start), start..=3).prop_map(|(start, end)| Interval::new(start, end).unwrap())
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    // Trace: TC-013, FR-004-AC-1, FR-001-AC-2, NFR-001-AC-2
    #[test]
    fn arbitrary_bounded_until_release_rules_match_the_corrected_oracle(
        kind in 0_u8..4,
        interval in bounded_interval(),
    ) {
        let (rule_id, input) = temporal_fixture(kind, interval);
        let rewritten = rewrite(&input, rule_id, RewriteOptions::default(), "source");
        prop_assert_eq!(rewritten.status, RewriteStatus::Normalized);
        prop_assert!(rewritten.steps.iter().any(|step| step.rule_id == rule_id));
        let output = rewritten.output.as_ref().unwrap();
        let conformance = check_equivalence(
            &input,
            output,
            rule_id,
            ConformanceOptions::default(),
        );
        prop_assert_eq!(conformance.status, ConformanceStatus::Equivalent);
    }
}
