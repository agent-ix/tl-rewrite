#![cfg(feature = "infinite-trace")]

use tl_mltl::infinite::EvaluationLimit;
use tl_rewrite::infinite::{
    check_infinite_rewrite, InfiniteConformanceReason, InfiniteConformanceStatus,
};
use tl_rewrite::{
    infinite_conformance_disposition, rewrite_infinite, DispositionMappingError, Fr341Disposition,
    RewriteOptions,
};
use tl_syntax::{
    FairnessPremisesDocument, InfiniteClock, InfiniteFormulaDocument, InfiniteNode,
    InfiniteNodeKind as Kind, LassoTraceDocument, NodeId, PartialValuation, PartialValue,
    PropositionId, SemanticProfile, TraceObservation, ValuationEntry,
};

const REVISION: &str = "infinite-conformance-fixture/v1";

fn graph() -> InfiniteFormulaDocument {
    InfiniteFormulaDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        NodeId(2),
        vec![
            InfiniteNode::new(Kind::False),
            InfiniteNode::new(Kind::Proposition {
                proposition: PropositionId(0),
            }),
            InfiniteNode::new(Kind::And {
                left: NodeId(0),
                right: NodeId(1),
            }),
        ],
    )
    .unwrap()
}

fn trace(value: PartialValue) -> LassoTraceDocument {
    let proposition = PropositionId(0);
    let propositions = vec![proposition];
    LassoTraceDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        "map".to_owned(),
        propositions.clone(),
        Vec::new(),
        vec![TraceObservation {
            position: 0,
            valuation: PartialValuation::new(
                "map".to_owned(),
                &propositions,
                vec![ValuationEntry { proposition, value }],
            )
            .unwrap(),
        }],
    )
    .unwrap()
}

// Trace: TC-069, FR-019-AC-3.
#[test]
fn fr019_conflict_is_classified_before_fold_can_hide_it() {
    let input = graph();
    let rewrite = rewrite_infinite(&input, None, RewriteOptions::default(), REVISION, 1_000_000);
    assert!(rewrite.succeeded());
    let check = check_infinite_rewrite(
        &input,
        None,
        &trace(PartialValue::Conflicting),
        &rewrite,
        0,
        REVISION,
        EvaluationLimit::default(),
    );
    assert_eq!(check.status, InfiniteConformanceStatus::NonConclusive);
    assert_eq!(
        check.reason,
        Some(InfiniteConformanceReason::ConflictingObservation)
    );
    assert!(check.before.is_none() && check.after.is_none());
}

// Trace: TC-069, FR-019-AC-3.
#[test]
fn fr019_complete_and_missing_evidence_compare_both_graphs() {
    let input = graph();
    let rewrite = rewrite_infinite(&input, None, RewriteOptions::default(), REVISION, 1_000_000);
    for value in [
        PartialValue::True,
        PartialValue::False,
        PartialValue::Missing,
    ] {
        let check = check_infinite_rewrite(
            &input,
            None,
            &trace(value),
            &rewrite,
            0,
            REVISION,
            EvaluationLimit::default(),
        );
        assert_eq!(check.status, InfiniteConformanceStatus::Equivalent);
        assert_eq!(check.reason, None);
        assert!(check.before.is_some() && check.after.is_some());
    }
}

// Trace: TC-074, FR-021-AC-2.
#[test]
fn fr021_provider_resource_failure_has_no_equivalence_credit() {
    let input = graph();
    let rewrite = rewrite_infinite(&input, None, RewriteOptions::default(), REVISION, 1_000_000);
    let check = check_infinite_rewrite(
        &input,
        None,
        &trace(PartialValue::True),
        &rewrite,
        0,
        REVISION,
        EvaluationLimit {
            max_nodes: 0,
            ..EvaluationLimit::default()
        },
    );
    assert_eq!(check.status, InfiniteConformanceStatus::NonConclusive);
    assert_eq!(
        check.reason,
        Some(InfiniteConformanceReason::ResourceIncomplete)
    );
    assert!(check.before.is_some() && check.after.is_some());
}

// Trace: TC-074, FR-021-AC-2.
#[test]
fn fr021_empty_fair_admission_has_no_equivalence_credit() {
    let input = graph();
    let fairness = FairnessPremisesDocument::new(
        &input,
        input.content_identity().unwrap(),
        InfiniteClock::EventPosition,
        vec![NodeId(0)],
    )
    .unwrap();
    let rewrite = rewrite_infinite(
        &input,
        Some(&fairness),
        RewriteOptions::default(),
        REVISION,
        1_000_000,
    );
    assert!(rewrite.succeeded());
    let check = check_infinite_rewrite(
        &input,
        Some(&fairness),
        &trace(PartialValue::True),
        &rewrite,
        0,
        REVISION,
        EvaluationLimit::default(),
    );
    assert_eq!(check.status, InfiniteConformanceStatus::NonConclusive);
    assert_eq!(
        check.reason,
        Some(InfiniteConformanceReason::EmptyFairAdmission)
    );
    assert!(check.before.is_some() && check.after.is_some());
}

// Trace: TC-075, NFR-005-AC-1.
#[test]
fn nfr005_mismatched_replay_refuses_before_provider() {
    let input = graph();
    let rewrite = rewrite_infinite(&input, None, RewriteOptions::default(), REVISION, 1_000_000);
    let check = check_infinite_rewrite(
        &input,
        None,
        &trace(PartialValue::True),
        &rewrite,
        0,
        "other-revision",
        EvaluationLimit::default(),
    );
    assert_eq!(check.status, InfiniteConformanceStatus::NonConclusive);
    assert_eq!(
        check.reason,
        Some(InfiniteConformanceReason::InvalidRewrite)
    );
    assert!(check.before.is_none() && check.after.is_none());
}

// Trace: TC-069, FR-019-AC-3.
#[test]
fn fr019_unknown_rule_record_cannot_reach_provider() {
    let input = graph();
    let mut rewrite =
        rewrite_infinite(&input, None, RewriteOptions::default(), REVISION, 1_000_000);
    assert!(!rewrite.steps.is_empty());
    rewrite.steps[0].rule_id = "unknown.infinite.rule".to_owned();
    let check = check_infinite_rewrite(
        &input,
        None,
        &trace(PartialValue::True),
        &rewrite,
        0,
        REVISION,
        EvaluationLimit::default(),
    );
    assert_eq!(check.status, InfiniteConformanceStatus::NonConclusive);
    assert_eq!(
        check.reason,
        Some(InfiniteConformanceReason::InvalidRewrite)
    );
    assert!(check.before.is_none() && check.after.is_none());
}

// Trace: TC-073, FR-021-AC-1.
#[test]
fn fr021_every_infinite_conformance_reason_has_a_typed_disposition() {
    let cases = [
        (
            InfiniteConformanceReason::InvalidRewrite,
            Fr341Disposition::Unsupported,
        ),
        (
            InfiniteConformanceReason::InvalidTrace,
            Fr341Disposition::Unsupported,
        ),
        (
            InfiniteConformanceReason::ConflictingObservation,
            Fr341Disposition::NoTemporalVerdict,
        ),
        (
            InfiniteConformanceReason::EmptyFairAdmission,
            Fr341Disposition::NoTemporalVerdict,
        ),
        (
            InfiniteConformanceReason::ResourceIncomplete,
            Fr341Disposition::ResourceIncomplete,
        ),
        (
            InfiniteConformanceReason::ProviderRefusal,
            Fr341Disposition::Unsupported,
        ),
        (
            InfiniteConformanceReason::ProviderFailure,
            Fr341Disposition::Failed,
        ),
    ];
    for (reason, expected) in cases {
        assert_eq!(
            infinite_conformance_disposition(
                InfiniteConformanceStatus::NonConclusive,
                Some(reason)
            ),
            Ok(expected),
        );
    }
    for status in [
        InfiniteConformanceStatus::Equivalent,
        InfiniteConformanceStatus::Mismatch,
    ] {
        assert_eq!(
            infinite_conformance_disposition(status, None),
            Ok(Fr341Disposition::NoTemporalVerdict),
        );
        assert_eq!(
            infinite_conformance_disposition(
                status,
                Some(InfiniteConformanceReason::ProviderFailure),
            ),
            Err(DispositionMappingError::UnexpectedReason),
        );
    }
    assert_eq!(
        infinite_conformance_disposition(InfiniteConformanceStatus::NonConclusive, None),
        Err(DispositionMappingError::MissingReason),
    );
}
