//! NFR-006: every declared limit is proved at its value and one past it, every
//! public status, reason, failure and error variant is produced through the
//! public API, and the Miri carve-outs are explicit.
//!
//! "Accepted" means not refused by that limit's own typed error. Where an owner
//! value cannot be reached by any input, the test says why and asserts the clamp
//! instead.

mod common;

use common::{document, proposition};
use tl_rewrite::{
    check_equivalence, replay_infinite, rewrite, rewrite_infinite, rewrite_with_context,
    BudgetKind, ConformanceOptions, ConformanceReason, ConformanceStatus, InfiniteRewriteFailure,
    InfiniteRewriteReport, RecordLimits, RecordReadErrorCode, RewriteBudgets, RewriteOptions,
    RewriteReport, RewriteStatus, MAX_INFINITE_REPORT_BYTES,
};
use tl_syntax::{
    FairnessPremisesDocument, FormulaDocument, InfiniteClock, InfiniteFormulaDocument,
    InfiniteNode, InfiniteNodeKind as InfKind, Interval, Node, NodeId, NodeKind,
    OwnedSignalDeclaration, PropositionBinding, PropositionId, SemanticProfile,
    SignalCatalogDocument, SignalDomain, SignalId,
};

/// The caller-lowered limit the Miri run substitutes for the owner value.
const MIRI_LOWERED: usize = 4096;

#[derive(Clone, Copy, Debug)]
enum Field {
    Iterations,
    Nodes,
    RuleApplications,
    WorkUnits,
}

const FIELDS: [Field; 4] = [
    Field::Iterations,
    Field::Nodes,
    Field::RuleApplications,
    Field::WorkUnits,
];

impl Field {
    fn kind(self) -> BudgetKind {
        match self {
            Self::Iterations => BudgetKind::Iterations,
            Self::Nodes => BudgetKind::Nodes,
            Self::RuleApplications => BudgetKind::RuleApplications,
            Self::WorkUnits => BudgetKind::WorkUnits,
        }
    }

    fn set(self, value: u64) -> RewriteBudgets {
        let mut budgets = RewriteBudgets::default();
        match self {
            Self::Iterations => budgets.max_iterations = u32::try_from(value).unwrap(),
            Self::Nodes => budgets.max_nodes = u32::try_from(value).unwrap(),
            Self::RuleApplications => budgets.max_rule_applications = value,
            Self::WorkUnits => budgets.max_work_units = value,
        }
        budgets
    }
}

/// What one rewrite attempt under a budget returned: success, or the budget kind
/// that refused it.
type Outcome = Result<(), BudgetKind>;

/// Asserts that, for every budget field, a budget equal to the need succeeds and
/// a budget one less is refused with the matching kind.
///
/// `needs` gives the measured need of the three fields a report records; the
/// node need is found by search because reports do not record it.
fn assert_budget_boundaries(run: &dyn Fn(RewriteBudgets) -> Outcome, needs: [u64; 3]) {
    let [iterations, applications, work] = needs;
    assert!(run(RewriteBudgets::default()).is_ok());
    let nodes = (0..=64_u64)
        .find(|nodes| run(Field::Nodes.set(*nodes)).is_ok())
        .expect("a small graph needs a small node budget");
    for (field, need) in [
        (Field::Iterations, iterations),
        (Field::Nodes, nodes),
        (Field::RuleApplications, applications),
        (Field::WorkUnits, work),
    ] {
        assert!(need > 0, "{field:?} must be exercised by the fixture");
        assert_eq!(run(field.set(need)), Ok(()), "{field:?} at its need");
        assert_eq!(
            run(field.set(need - 1)),
            Err(field.kind()),
            "{field:?} one less"
        );
    }
    assert_eq!(FIELDS.len(), 4);
}

fn finite_outcome(report: &RewriteReport) -> Outcome {
    match report.status {
        RewriteStatus::Unchanged | RewriteStatus::Normalized => Ok(()),
        RewriteStatus::BudgetExhausted => Err(report.exhausted_budget.unwrap()),
        other => panic!("unexpected {other:?}: {:?}", report.detail),
    }
}

fn options(budgets: RewriteBudgets) -> RewriteOptions {
    RewriteOptions {
        budgets,
        ..RewriteOptions::default()
    }
}

fn reducible() -> FormulaDocument {
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

fn past_reducible() -> FormulaDocument {
    // not (since (not p0) (not p1)): the triggered dual folds.
    let interval = Interval::new(0, 2).unwrap();
    let nodes = vec![
        proposition(0),
        proposition(1),
        Node::new(NodeKind::Not { operand: NodeId(0) }),
        Node::new(NodeKind::Not { operand: NodeId(1) }),
        Node::new(NodeKind::Since {
            interval,
            left: NodeId(2),
            right: NodeId(3),
        }),
        Node::new(NodeKind::Not { operand: NodeId(4) }),
    ];
    FormulaDocument::new_v2(SemanticProfile::OriginCompleteHistoryV1, NodeId(5), nodes).unwrap()
}

fn infinite_double_negation() -> InfiniteFormulaDocument {
    InfiniteFormulaDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        NodeId(2),
        [
            InfKind::Proposition {
                proposition: PropositionId(0),
            },
            InfKind::Not { operand: NodeId(0) },
            InfKind::Not { operand: NodeId(1) },
        ]
        .into_iter()
        .map(InfiniteNode::new)
        .collect(),
    )
    .unwrap()
}

// Trace: TC-081, NFR-006-AC-5
#[test]
fn a_deserialized_fairness_root_outside_the_graph_is_refused() {
    let input = infinite_double_negation();
    let identity = input.content_identity().unwrap();
    let fairness = FairnessPremisesDocument::new(
        &input,
        identity,
        InfiniteClock::EventPosition,
        vec![NodeId(0)],
    )
    .unwrap();
    // The serde path checks roots for order and duplicates, not against the graph,
    // so a document with a matching graph identity can name a node that is absent.
    let mut wire = serde_json::to_value(&fairness).unwrap();
    wire["roots"] = serde_json::json!([999]);
    let forged: FairnessPremisesDocument = serde_json::from_value(wire).unwrap();
    let report = rewrite_infinite(
        &input,
        Some(&forged),
        RewriteOptions::default(),
        "source",
        1_000_000,
    );
    assert!(!report.succeeded());
    assert_eq!(
        report.failure,
        Some(InfiniteRewriteFailure::IdentityMismatch)
    );
    let accepted = rewrite_infinite(
        &input,
        Some(&fairness),
        RewriteOptions::default(),
        "source",
        1_000_000,
    );
    assert!(accepted.succeeded());
    assert!(!replay_infinite(&input, Some(&forged), &accepted, "source"));
}

fn infinite_outcome(report: &InfiniteRewriteReport) -> Outcome {
    if report.succeeded() {
        return Ok(());
    }
    match report.failure {
        Some(InfiniteRewriteFailure::Budget(kind)) => Err(kind),
        other => panic!("unexpected {other:?}"),
    }
}

// Trace: TC-077, NFR-006-AC-1
#[test]
fn finite_budgets_succeed_at_the_need_and_exhaust_one_less() {
    let input = reducible();
    let run = |budgets| finite_outcome(&rewrite(&input, "finite", options(budgets), "source"));
    let baseline = rewrite(&input, "finite", RewriteOptions::default(), "source");
    assert_eq!(baseline.status, RewriteStatus::Normalized);
    assert_budget_boundaries(
        &run,
        [
            u64::from(baseline.iterations),
            baseline.rule_applications,
            baseline.work_units,
        ],
    );
}

// Trace: TC-077, NFR-006-AC-1
#[test]
fn past_budgets_succeed_at_the_need_and_exhaust_one_less() {
    let input = past_reducible();
    let run = |budgets| past_outcome(&input, budgets);
    fn past_outcome(input: &FormulaDocument, budgets: RewriteBudgets) -> Outcome {
        finite_outcome(&rewrite(input, "past", options(budgets), "source"))
    }
    let baseline = rewrite(&input, "past", RewriteOptions::default(), "source");
    assert_eq!(baseline.status, RewriteStatus::Normalized);
    assert_budget_boundaries(
        &run,
        [
            u64::from(baseline.iterations),
            baseline.rule_applications,
            baseline.work_units,
        ],
    );
}

// Trace: TC-077, NFR-006-AC-1
#[test]
fn infinite_budgets_succeed_at_the_need_and_exhaust_one_less() {
    let input = infinite_double_negation();
    let attempt = |budgets| rewrite_infinite(&input, None, options(budgets), "source", 1_000_000);
    let baseline = attempt(RewriteBudgets::default());
    assert!(baseline.succeeded());
    let run = |budgets| infinite_outcome(&attempt(budgets));
    assert_budget_boundaries(
        &run,
        [
            u64::from(baseline.iterations),
            u64::try_from(baseline.steps.len()).unwrap(),
            baseline.work_units,
        ],
    );
}

// Trace: TC-078, NFR-006-AC-2
#[test]
fn infinite_report_bytes_accept_the_limit_and_refuse_one_past() {
    // The report embeds its own byte ceiling, so the exact size depends on the
    // digits of the ceiling. Search the narrow window around the unconstrained
    // size for the smallest ceiling that holds, then check its predecessor.
    let input = infinite_double_negation();
    let attempt =
        |ceiling| rewrite_infinite(&input, None, RewriteOptions::default(), "source", ceiling);
    let unconstrained = serde_json::to_vec(&attempt(1_000_000)).unwrap().len();
    // Native and Miri both run the boundary at the caller-lowered ceiling: a
    // report of MAX_INFINITE_REPORT_BYTES bytes would need a graph orders of
    // magnitude beyond any budget a test can spend.
    let need = (unconstrained - 8..=unconstrained)
        .find(|ceiling| attempt(*ceiling).succeeded())
        .expect("a ceiling just under the unconstrained size holds");
    assert!(attempt(need).succeeded());
    let refused = attempt(need - 1);
    assert!(!refused.succeeded());
    assert_eq!(refused.failure, Some(InfiniteRewriteFailure::ReportBytes));
    assert!(refused.output.is_none());

    // A caller value above the owner constant is clamped to it.
    assert_eq!(
        attempt(usize::MAX).max_report_bytes,
        MAX_INFINITE_REPORT_BYTES
    );
}

fn spaces(count: usize) -> Vec<u8> {
    vec![b' '; count]
}

fn record_error(bytes: &[u8], limits: RecordLimits) -> tl_rewrite::RecordReadError {
    RewriteReport::from_json_bytes(bytes, limits).unwrap_err()
}

// Trace: TC-079, NFR-006-AC-3
#[test]
fn record_document_bytes_accept_the_limit_and_refuse_one_past() {
    // Native: the owner value, reached by clamping a larger caller value. Miri: a
    // caller-lowered ceiling on the same code path.
    let limit = if cfg!(miri) {
        MIRI_LOWERED
    } else {
        RecordLimits::OWNER_MAXIMA.document_bytes
    };
    let limits = RecordLimits {
        document_bytes: if cfg!(miri) { limit } else { usize::MAX },
        ..RecordLimits::OWNER_MAXIMA
    };
    // Blank input passes every ceiling and is then refused as malformed.
    assert_eq!(
        record_error(&spaces(limit), limits).code(),
        RecordReadErrorCode::Malformed
    );
    let past = record_error(&spaces(limit + 1), limits);
    assert_eq!(past.code(), RecordReadErrorCode::ResourceLimit);
    assert_eq!(past.resource(), "documentBytes");
    assert_eq!(
        (past.actual(), past.limit()),
        (Some(limit + 1), Some(limit))
    );
}

// Trace: TC-079, NFR-006-AC-3
#[test]
fn record_json_depth_accepts_the_limit_and_refuses_one_past() {
    let owner = RecordLimits::OWNER_MAXIMA.json_depth;
    let nested = |depth: usize| format!("{}{}", "[".repeat(depth), "]".repeat(depth)).into_bytes();
    let limits = RecordLimits {
        json_depth: usize::MAX,
        ..RecordLimits::OWNER_MAXIMA
    };
    assert_ne!(
        record_error(&nested(owner), limits).code(),
        RecordReadErrorCode::ResourceLimit
    );
    // serde_json refuses nesting past its own recursion limit, which equals the
    // owner depth, before the owner's measurement runs: one past the owner depth
    // is malformed, so the typed depth refusal is proved at a lowered ceiling.
    assert_eq!(
        record_error(&nested(owner + 1), limits).code(),
        RecordReadErrorCode::Malformed
    );
    let lowered = RecordLimits {
        json_depth: 8,
        ..RecordLimits::OWNER_MAXIMA
    };
    assert_ne!(
        record_error(&nested(8), lowered).code(),
        RecordReadErrorCode::ResourceLimit
    );
    let past = record_error(&nested(9), lowered);
    assert_eq!(past.code(), RecordReadErrorCode::ResourceLimit);
    assert_eq!(past.resource(), "jsonDepth");
}

// Trace: TC-079, NFR-006-AC-3
#[test]
fn record_string_bytes_accept_the_limit_and_refuse_one_past() {
    let owner = RecordLimits::OWNER_MAXIMA.string_bytes;
    let limits = RecordLimits {
        string_bytes: usize::MAX,
        ..RecordLimits::OWNER_MAXIMA
    };
    let quoted = |length: usize| format!("\"{}\"", "a".repeat(length)).into_bytes();
    assert_ne!(
        record_error(&quoted(owner), limits).code(),
        RecordReadErrorCode::ResourceLimit
    );
    let past = record_error(&quoted(owner + 1), limits);
    assert_eq!(past.code(), RecordReadErrorCode::ResourceLimit);
    assert_eq!(past.resource(), "stringBytes");
    assert_eq!(
        (past.actual(), past.limit()),
        (Some(owner + 1), Some(owner))
    );
}

fn conjunction(propositions: u32) -> FormulaDocument {
    let mut nodes: Vec<Node> = (0..propositions).map(proposition).collect();
    let mut accumulated = NodeId(0);
    for index in 1..propositions {
        nodes.push(Node::new(NodeKind::And {
            left: accumulated,
            right: NodeId(index),
        }));
        accumulated = NodeId(u32::try_from(nodes.len() - 1).unwrap());
    }
    document(SemanticProfile::ClosedTraceV1, nodes)
}

fn always_true(end: u32) -> FormulaDocument {
    document(
        SemanticProfile::ClosedTraceV1,
        vec![
            Node::new(NodeKind::True),
            Node::new(NodeKind::Globally {
                interval: Interval::new(0, end).unwrap(),
                operand: NodeId(0),
            }),
        ],
    )
}

fn compare(
    formula: &FormulaDocument,
    options: ConformanceOptions,
) -> tl_rewrite::ConformanceReport {
    check_equivalence(formula, formula, "boundary", options)
}

fn generous() -> ConformanceOptions {
    ConformanceOptions {
        max_propositions: 8,
        max_horizon: 8,
        max_traces: 1 << 20,
    }
}

// Trace: TC-080, NFR-006-AC-4
#[test]
fn conformance_proposition_limit_accepts_the_limit_and_refuses_one_past() {
    let options = ConformanceOptions {
        max_propositions: 3,
        ..generous()
    };
    assert_eq!(
        compare(&conjunction(3), options).status,
        ConformanceStatus::Equivalent
    );
    let past = compare(&conjunction(4), options);
    assert_eq!(past.status, ConformanceStatus::NonConclusive);
    assert_eq!(past.reason, Some(ConformanceReason::PropositionLimit));
}

// Trace: TC-080, NFR-006-AC-4
#[test]
fn conformance_horizon_limit_accepts_the_limit_and_refuses_one_past() {
    let options = ConformanceOptions {
        max_horizon: 3,
        ..generous()
    };
    let at = compare(&always_true(3), options);
    assert_eq!(at.horizon, Some(3));
    assert_eq!(at.status, ConformanceStatus::Equivalent);
    let past = compare(&always_true(4), options);
    assert_eq!(past.reason, Some(ConformanceReason::HorizonLimit));
    assert_eq!(past.status, ConformanceStatus::NonConclusive);
}

// Trace: TC-080, NFR-006-AC-4
#[test]
fn conformance_trace_limit_accepts_the_limit_and_refuses_one_past() {
    let formula = conjunction(2);
    let at = compare(
        &formula,
        ConformanceOptions {
            max_traces: 4,
            ..generous()
        },
    );
    assert_eq!(at.total_traces, Some(4));
    assert_eq!(at.status, ConformanceStatus::Equivalent);
    let past = compare(
        &formula,
        ConformanceOptions {
            max_traces: 3,
            ..generous()
        },
    );
    assert_eq!(past.reason, Some(ConformanceReason::TraceDomainLimit));
    assert_eq!(past.status, ConformanceStatus::NonConclusive);
}

fn catalog_binding(proposition: u32) -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        vec![OwnedSignalDeclaration::new(
            SignalId(10),
            "brake".into(),
            SignalDomain::Boolean,
        )],
        vec![PropositionBinding::new(
            PropositionId(proposition),
            SignalId(10),
        )],
    )
    .unwrap()
}

// Trace: TC-081, NFR-006-AC-5
#[test]
fn every_rewrite_status_is_returned_by_a_rewriter() {
    let mut seen = std::collections::BTreeSet::new();
    let mut note = |status: RewriteStatus| seen.insert(format!("{status:?}"));
    let plain = document(SemanticProfile::ClosedTraceV1, vec![proposition(0)]);
    note(rewrite(&plain, "unchanged", RewriteOptions::default(), "s").status);
    note(rewrite(&reducible(), "normalized", RewriteOptions::default(), "s").status);
    note(rewrite(&reducible(), "budget", options(Field::Nodes.set(0)), "s").status);
    let online = document(SemanticProfile::OnlinePrefixV1, vec![proposition(0)]);
    note(rewrite(&online, "profile", RewriteOptions::default(), "s").status);
    note(
        rewrite_with_context(
            &plain,
            "binding",
            RewriteOptions::default(),
            "s",
            &catalog_binding(1),
            None,
        )
        .status,
    );
    // The infinite path maps a foreign identity to InvalidInput.
    let mismatched = rewrite_infinite(
        &infinite_double_negation(),
        None,
        RewriteOptions::default(),
        "",
        1_000_000,
    );
    note(mismatched.status);
    let expected: std::collections::BTreeSet<String> = [
        "Unchanged",
        "Normalized",
        "BudgetExhausted",
        "UnsupportedProfile",
        "UnresolvedBinding",
        "InvalidInput",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    assert_eq!(seen, expected);
}

// Trace: TC-081, NFR-006-AC-5
#[test]
fn every_conformance_reason_is_returned_by_a_comparison() {
    use tl_rewrite::check_equivalence_with_context;
    let mut seen = std::collections::BTreeSet::new();
    let mut note = |report: tl_rewrite::ConformanceReport| {
        seen.insert(format!(
            "{:?}",
            report.reason.expect("a refusal carries a reason")
        ))
    };
    let closed = conjunction(1);
    let online = document(SemanticProfile::OnlinePrefixV1, vec![proposition(0)]);
    note(check_equivalence(&online, &online, "profile", generous()));
    note(compare(
        &conjunction(4),
        ConformanceOptions {
            max_propositions: 3,
            ..generous()
        },
    ));
    note(compare(
        &always_true(4),
        ConformanceOptions {
            max_horizon: 3,
            ..generous()
        },
    ));
    note(compare(
        &conjunction(2),
        ConformanceOptions {
            max_traces: 3,
            ..generous()
        },
    ));
    let other = document(SemanticProfile::ClosedTraceV1, vec![proposition(7)]);
    note(check_equivalence_with_context(
        &closed,
        &closed,
        "original-binding",
        generous(),
        &catalog_binding(7),
        None,
    ));
    note(check_equivalence_with_context(
        &other,
        &closed,
        "rewritten-binding",
        generous(),
        &catalog_binding(7),
        None,
    ));
    let expected: std::collections::BTreeSet<String> = [
        "UnsupportedProfile",
        "PropositionLimit",
        "HorizonLimit",
        "TraceDomainLimit",
        "OriginalBinding",
        "RewrittenBinding",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    // `EvaluatorError` needs a graph deeper than the evaluator accepts; see the
    // dedicated test below.
    assert_eq!(seen, expected);
}

// Trace: TC-081, NFR-006-AC-5
#[test]
#[cfg_attr(
    miri,
    ignore = "builds a graph deeper than the evaluator accepts, which is too slow to interpret"
)]
fn the_evaluator_error_reason_is_returned_for_an_over_deep_graph() {
    let mut nodes = vec![proposition(0)];
    for operand in 0..513 {
        nodes.push(Node::new(NodeKind::Not {
            operand: NodeId(operand),
        }));
    }
    let deep = document(SemanticProfile::ClosedTraceV1, nodes);
    let report = check_equivalence(
        &deep,
        &deep,
        "evaluator-depth",
        ConformanceOptions::default(),
    );
    assert_eq!(report.reason, Some(ConformanceReason::EvaluatorError));
}

// Trace: TC-081, NFR-006-AC-5
#[test]
fn the_past_unsupported_profile_reason_is_returned_for_a_closed_trace_formula() {
    use tl_mltl::{ClockBinding, PositionHistoryDocument, PositionObservation};
    use tl_rewrite::{check_past_equivalence, PastConformanceReason, PastEvaluationContext};
    let history = PositionHistoryDocument::new(
        "history:robustness".to_owned(),
        1,
        0,
        0,
        Some(ClockBinding::EventPosition),
        vec![PositionObservation::new(0, vec![PropositionId(0)], None)],
    )
    .unwrap();
    let formula = conjunction(1);
    let context = PastEvaluationContext {
        formula: &formula,
        formula_id: "closed",
        history: &history,
        anchor: 0,
        proposition_map_id: "map",
    };
    let report = check_past_equivalence(
        &context,
        &context,
        "profile",
        tl_mltl::PastEvaluationLimits::default(),
    );
    assert_eq!(report.status, ConformanceStatus::NonConclusive);
    assert_eq!(
        report.reason,
        Some(PastConformanceReason::UnsupportedProfile)
    );
}

// Trace: TC-082, NFR-006-AC-6
#[test]
fn miri_target_runs_in_the_ci_composite_on_the_pinned_nightly() {
    let makefile = include_str!("../Makefile");
    let composite = makefile
        .split("\nci:")
        .nth(1)
        .and_then(|rest| rest.split("\n\n").next())
        .expect("Makefile declares the ci composite");
    assert!(
        composite.split_whitespace().any(|word| word == "miri"),
        "{composite}"
    );
    assert!(makefile.contains("MIRI_TOOLCHAIN ?= nightly-"));
    assert!(makefile.contains("MIRIFLAGS"));
    assert!(makefile.contains("miri test"));
}
