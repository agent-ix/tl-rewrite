mod common;

use std::{fs, path::PathBuf};

use common::{document, proposition, west_document};
use serde::Deserialize;
use tl_rewrite::{
    check_equivalence, rewrite, ConformanceOptions, ConformanceReason, ConformanceStatus,
    RewriteOptions, RewriteStatus, TL_MLTL_REVISION, WEST_REVISION,
};
use tl_syntax::{Interval, Node, NodeId, NodeKind, SemanticProfile};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WestManifest {
    schema_version: String,
    upstream_revision: String,
    selected_cases: Vec<WestCase>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WestCase {
    id: String,
    line: usize,
    source: String,
    expected_rule: String,
}

// Trace: TC-013, FR-004-AC-1
#[test]
fn supported_pair_is_exhaustively_equivalent_over_horizon() {
    let interval = Interval::new(0, 2).unwrap();
    let input = document(
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
    );
    let rewritten = rewrite(&input, "until", RewriteOptions::default(), "source")
        .output
        .unwrap();
    let report = check_equivalence(
        &input,
        &rewritten,
        "until-equivalence",
        ConformanceOptions::default(),
    );
    assert_eq!(report.status, ConformanceStatus::Equivalent);
    assert_eq!(report.horizon, Some(2));
    assert_eq!(report.trace_length, Some(3));
    assert_eq!(report.total_traces, Some(8));
    assert_eq!(report.traces_checked, 8);
    assert_eq!(report.evaluator_revision, TL_MLTL_REVISION);
}

// Trace: TC-014, FR-004-AC-1
#[test]
fn mismatch_retains_first_deterministic_counterexample() {
    let original = document(SemanticProfile::ClosedTraceV1, vec![proposition(0)]);
    let changed = document(
        SemanticProfile::ClosedTraceV1,
        vec![
            proposition(0),
            Node::new(NodeKind::Not { operand: NodeId(0) }),
        ],
    );
    let report = check_equivalence(
        &original,
        &changed,
        "known-mismatch",
        ConformanceOptions::default(),
    );
    assert_eq!(report.status, ConformanceStatus::Mismatch);
    assert_eq!(report.traces_checked, 1);
    assert_eq!(report.counterexample, Some(vec![vec![]]));
    assert_ne!(report.original_verdict, report.rewritten_verdict);
}

// Trace: TC-015, FR-004-AC-2, NFR-001-AC-2
#[test]
fn unsupported_profiles_and_domain_limits_are_nonconclusive() {
    let online = document(SemanticProfile::OnlinePrefixV1, vec![proposition(0)]);
    let report = check_equivalence(&online, &online, "online", ConformanceOptions::default());
    assert_eq!(report.status, ConformanceStatus::NonConclusive);
    assert_eq!(report.reason, Some(ConformanceReason::UnsupportedProfile));

    let closed = document(SemanticProfile::ClosedTraceV1, vec![proposition(0)]);
    let report = check_equivalence(
        &closed,
        &closed,
        "props",
        ConformanceOptions {
            max_propositions: 0,
            ..ConformanceOptions::default()
        },
    );
    assert_eq!(report.reason, Some(ConformanceReason::PropositionLimit));
    let report = check_equivalence(
        &closed,
        &closed,
        "traces",
        ConformanceOptions {
            max_traces: 1,
            ..ConformanceOptions::default()
        },
    );
    assert_eq!(report.reason, Some(ConformanceReason::TraceDomainLimit));

    let large = document(
        SemanticProfile::ClosedTraceV1,
        vec![
            Node::new(NodeKind::True),
            Node::new(NodeKind::Future {
                interval: Interval::new(0, 100_000).unwrap(),
                operand: NodeId(0),
            }),
        ],
    );
    let report = check_equivalence(
        &large,
        &large,
        "materialization",
        ConformanceOptions {
            max_horizon: 100_000,
            ..ConformanceOptions::default()
        },
    );
    assert_eq!(report.reason, Some(ConformanceReason::TraceDomainLimit));
}

// Trace: TC-016, FR-004-AC-3, StR-002-VC-2, NFR-002-AC-2
#[test]
fn pinned_west_subset_rewrites_and_is_exhaustively_equivalent() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus/west-v1");
    let manifest: WestManifest =
        serde_json::from_slice(&fs::read(root.join("manifest.json")).unwrap()).unwrap();
    let source = fs::read_to_string(root.join("formulas_d1.txt")).unwrap();
    let source_lines = source.lines().collect::<Vec<_>>();
    assert_eq!(manifest.schema_version, "tl-rewrite.west-corpus/v1");
    assert_eq!(manifest.upstream_revision, WEST_REVISION);
    assert_eq!(manifest.selected_cases.len(), 10);

    for case in manifest.selected_cases {
        assert_eq!(source_lines[case.line - 1], case.source);
        let input = west_document(&case.id);
        let rewritten = rewrite(&input, case.id.clone(), RewriteOptions::default(), "source");
        assert_eq!(rewritten.status, RewriteStatus::Normalized, "{}", case.id);
        assert!(
            rewritten
                .steps
                .iter()
                .any(|step| step.rule_id == case.expected_rule),
            "{} did not apply {}",
            case.id,
            case.expected_rule
        );
        let conformance = check_equivalence(
            &input,
            rewritten.output.as_ref().unwrap(),
            case.id.clone(),
            ConformanceOptions::default(),
        );
        assert_eq!(
            conformance.status,
            ConformanceStatus::Equivalent,
            "{}",
            case.id
        );
    }
}
