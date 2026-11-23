//! Solver-backed bounded equivalence behind the non-default `solver` feature.
//!
//! [`check_equivalence_with_solver`] decides the same question as
//! [`crate::check_equivalence`] (do the two closed-trace formulas agree on every
//! trace of length `horizon + 1`?) without enumerating traces. It builds the
//! miter `¬(original ↔ rewritten)` as one tl-syntax graph, asks tl-solve whether
//! it is satisfiable at that exact trace length, and maps the answer:
//!
//! | tl-solve | Report |
//! |---|---|
//! | `unsatisfiable` (with a basis) | `equivalent`, basis and solver identity recorded |
//! | `satisfiable` with a trace on which tl-rewrite's own `evaluate_closed` re-check gives different verdicts | `mismatch` with that trace and both verdicts |
//! | `satisfiable` with a trace on which the verdicts agree | `non_conclusive` / `counterexample_not_confirmed` (never `mismatch`) |
//! | re-check stopped on its budget | `non_conclusive` / `recheck_limit` |
//! | any `inconclusive`, `refused` or `failed` result | `non_conclusive` with the mirrored reason |
//!
//! The enumeration path is not changed and stays the default evidence; this
//! path only adds evidence. UNBUILT: never compiled (TL-DECISIONS-2026-11-23).
//!
//! Implements: FR-091

use serde::{Deserialize, Serialize};
use tl_mltl::{analyze_horizon, evaluate_closed, EvaluationError, EvaluationLimits, TruthValue};
use tl_solve::wire::result::{LimitsRecord, SolverRecord};
use tl_solve::{
    check_satisfiable, Basis, Disposition, EncodeLimits, FrameRequest, ReasonCode, RecheckLimits,
    ResultDocument, SatBackend, SatRequest, SolveLimits, Witness,
};
use tl_syntax::{
    Formula, FormulaDocument, Node, NodeId, NodeKind, PropositionId, SemanticProfile,
};

use crate::{engine::remap, hash::sha256_json};

/// Wire schema of [`SolverConformanceReport`].
pub const SOLVER_CONFORMANCE_SCHEMA: &str = "tl-rewrite.solver-conformance/v1";

/// Formula identity tl-solve records for the miter.
const MITER_FORMULA_ID: &str = "tl-rewrite.solver-miter";

/// Limits of one solver-backed comparison.
///
/// SPEC-GAP: FR-091 lists `SolverConformanceOptions { limits, recheck_limits,
/// backend }`. A `SatBackend` is a mutable trait object, so it is passed to
/// [`check_equivalence_with_solver`] as its own argument; `limits` is split
/// into tl-solve's two limit types.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SolverConformanceOptions {
    /// tl-solve encoder limits (trace length, variables, clauses, nodes).
    pub encode_limits: EncodeLimits,
    /// tl-solve solver limits (conflicts, wall time, memory).
    pub solve_limits: SolveLimits,
    /// Node-evaluation budget of every `evaluate_closed` re-check, both
    /// tl-solve's own and tl-rewrite's. Default: tl-mltl's evaluator default
    /// (1,000,000), the value tl-solve FR-009 uses.
    pub recheck_limits: RecheckLimits,
}

/// Typed reason a solver-backed comparison was not conclusive.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolverConformanceReason {
    /// Profiles differed or were not closed-trace v1.
    UnsupportedProfile,
    /// tl-mltl's horizon analysis refused a formula.
    HorizonError,
    /// The miter graph could not be built or failed tl-syntax validation
    /// (node, depth or identity limits).
    MiterRefused,
    /// tl-solve's encoder refused the miter; `reasonDetail` names its code.
    EncodeRefused,
    /// The backend does not declare a requested capability (refused backend).
    CapabilityUnsupported,
    /// The solver's conflict limit fired.
    ConflictLimit,
    /// The solver's wall-clock limit fired.
    TimeLimit,
    /// The solver's memory limit fired.
    MemoryLimit,
    /// A witness re-check (tl-solve's or tl-rewrite's) stopped on its budget.
    RecheckLimit,
    /// The backend returned a model that violates the CNF.
    InvalidModel,
    /// tl-solve's own re-check rejected the decoded witness.
    WitnessRejected,
    /// An evaluator error other than a budget stop, or a pending verdict.
    RecheckError,
    /// The backend failed; `reasonDetail` names the failure kind.
    BackendFailure,
    /// tl-solve returned a trace on which tl-rewrite's re-check found equal
    /// verdicts: a solver or encoder fault, never a mismatch (FR-091-AC-3).
    CounterexampleNotConfirmed,
    /// A tl-solve result whose fields contradict its disposition.
    MalformedSolverResult,
}

/// Versioned solver-backed comparison report (`tl-rewrite.solver-conformance/v1`).
///
/// The existing `ConformanceReport` and `tl-rewrite.conformance/v1` are not
/// modified by this feature.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SolverConformanceReport {
    /// Wire schema identity.
    pub schema_version: String,
    /// Caller-provided comparison identity.
    pub comparison_id: String,
    /// Canonical original formula digest.
    pub original_sha256: String,
    /// Canonical rewritten formula digest.
    pub rewritten_sha256: String,
    /// Semantic profile of the original formula.
    pub semantic_profile: String,
    /// Maximum formula lookahead.
    pub horizon: Option<u64>,
    /// Trace length decided (`horizon + 1`, as in enumeration).
    pub trace_length: Option<u64>,
    /// Result classification.
    pub status: crate::ConformanceStatus,
    /// Reason when non-conclusive.
    pub reason: Option<SolverConformanceReason>,
    /// Bounded detail from tl-solve's reason, when it gave one.
    pub reason_detail: Option<String>,
    /// tl-solve basis of an `equivalent` answer.
    pub basis: Option<Basis>,
    /// Solver identity of an `equivalent` or `mismatch` answer.
    pub solver: Option<SolverRecord>,
    /// Limits tl-solve recorded, when it ran.
    pub solver_limits: Option<LimitsRecord>,
    /// Node-evaluation budget of tl-rewrite's own re-check.
    pub recheck_node_evaluations: u64,
    /// Re-checked counterexample (true propositions per instant), on mismatch.
    pub counterexample: Option<Vec<Vec<u32>>>,
    /// A returned trace that failed tl-rewrite's re-check, kept for diagnosis.
    pub diagnostic_trace: Option<Vec<Vec<u32>>>,
    /// Original verdict on the counterexample or diagnostic trace.
    pub original_verdict: Option<TruthValue>,
    /// Rewritten verdict on the counterexample or diagnostic trace.
    pub rewritten_verdict: Option<TruthValue>,
    /// Scope boundary of the evidence.
    pub limitation: String,
}

fn report_base(
    original: &FormulaDocument,
    rewritten: &FormulaDocument,
    comparison_id: String,
    options: &SolverConformanceOptions,
) -> SolverConformanceReport {
    SolverConformanceReport {
        schema_version: SOLVER_CONFORMANCE_SCHEMA.to_owned(),
        comparison_id,
        original_sha256: sha256_json(&original.semantic_view()),
        rewritten_sha256: sha256_json(&rewritten.semantic_view()),
        semantic_profile: original.semantic_profile().as_str().to_owned(),
        horizon: None,
        trace_length: None,
        status: crate::ConformanceStatus::NonConclusive,
        reason: None,
        reason_detail: None,
        basis: None,
        solver: None,
        solver_limits: None,
        recheck_node_evaluations: options.recheck_limits.max_node_evaluations,
        counterexample: None,
        diagnostic_trace: None,
        original_verdict: None,
        rewritten_verdict: None,
        limitation: "solver-backed agreement covers exactly the traces of length horizon + 1 under the declared closed-trace profile, the same domain as enumeration; an equivalent answer rests on the tl-solve encoder and the recorded basis, and a mismatch is reported only after tl-mltl re-evaluated both formulas on the trace; this does not prove arbitrary rewrite schemas or qualify a consuming tool".to_owned(),
    }
}

fn non_conclusive(
    mut report: SolverConformanceReport,
    reason: SolverConformanceReason,
    detail: Option<String>,
) -> SolverConformanceReport {
    report.status = crate::ConformanceStatus::NonConclusive;
    report.reason = Some(reason);
    report.reason_detail = detail.filter(|text| !text.is_empty());
    report
}

/// Profile and horizon checks shared by the solving entry point and the
/// result-classification seam. `Ok` carries the trace length.
fn prepare(
    original: &FormulaDocument,
    rewritten: &FormulaDocument,
    comparison_id: String,
    options: &SolverConformanceOptions,
) -> Result<(SolverConformanceReport, u64), SolverConformanceReport> {
    let mut report = report_base(original, rewritten, comparison_id, options);
    if original.semantic_profile() != rewritten.semantic_profile()
        || original.semantic_profile() != SemanticProfile::ClosedTraceV1
    {
        return Err(non_conclusive(
            report,
            SolverConformanceReason::UnsupportedProfile,
            None,
        ));
    }
    let (Ok(original_formula), Ok(rewritten_formula)) = (original.validate(), rewritten.validate())
    else {
        return Err(non_conclusive(
            report,
            SolverConformanceReason::HorizonError,
            None,
        ));
    };
    let original_horizon = match analyze_horizon(original_formula, "original") {
        Ok(value) => value.lookahead,
        Err(_) => {
            return Err(non_conclusive(
                report,
                SolverConformanceReason::HorizonError,
                None,
            ))
        }
    };
    let rewritten_horizon = match analyze_horizon(rewritten_formula, "rewritten") {
        Ok(value) => value.lookahead,
        Err(_) => {
            return Err(non_conclusive(
                report,
                SolverConformanceReason::HorizonError,
                None,
            ))
        }
    };
    let horizon = original_horizon.max(rewritten_horizon);
    report.horizon = Some(horizon);
    let Some(trace_length) = horizon.checked_add(1) else {
        return Err(non_conclusive(
            report,
            SolverConformanceReason::HorizonError,
            None,
        ));
    };
    report.trace_length = Some(trace_length);
    Ok((report, trace_length))
}

fn node_id(index: usize) -> Option<NodeId> {
    u32::try_from(index).ok().map(NodeId)
}

/// The miter `¬(original ↔ rewritten)` as one closed-trace graph: the
/// original's nodes, then the rewritten's nodes shifted past them, then the
/// equivalence and its negation. Spans are dropped (they are not semantic).
fn miter_document(
    original: &FormulaDocument,
    rewritten: &FormulaDocument,
) -> Option<FormulaDocument> {
    let mut nodes: Vec<Node> = original
        .nodes()
        .iter()
        .map(|node| Node::new(node.kind))
        .collect();
    let offset = nodes.len();
    let mut shifted = Vec::with_capacity(rewritten.nodes().len());
    for index in 0..rewritten.nodes().len() {
        shifted.push(node_id(offset.checked_add(index)?)?);
    }
    for node in rewritten.nodes() {
        nodes.push(Node::new(remap(node.kind, &shifted)?));
    }
    let left = original.root();
    let right = *shifted.get(usize::try_from(rewritten.root().0).ok()?)?;
    nodes.push(Node::new(NodeKind::Equivalent { left, right }));
    let equivalent = node_id(nodes.len() - 1)?;
    nodes.push(Node::new(NodeKind::Not {
        operand: equivalent,
    }));
    let root = node_id(nodes.len() - 1)?;
    FormulaDocument::new(SemanticProfile::ClosedTraceV1, root, nodes).ok()
}

/// Outcome of tl-rewrite's own re-check of one formula.
enum Verdict {
    Value(TruthValue),
    Budget,
    Error,
}

fn verdict(
    formula: Formula<'_>,
    formula_id: &str,
    instants: &[Vec<PropositionId>],
    limits: RecheckLimits,
) -> Verdict {
    let evaluation = EvaluationLimits {
        max_node_evaluations: limits.max_node_evaluations,
        ..EvaluationLimits::default()
    };
    match evaluate_closed(
        formula,
        formula_id,
        instants,
        "tl-rewrite.solver-counterexample".to_owned(),
        evaluation,
    ) {
        Ok(report) => match report.verdict {
            TruthValue::True | TruthValue::False => Verdict::Value(report.verdict),
            TruthValue::Pending => Verdict::Error,
        },
        Err(EvaluationError::WorkLimitExceeded { .. }) => Verdict::Budget,
        Err(_) => Verdict::Error,
    }
}

fn plain_trace(instants: &[Vec<PropositionId>]) -> Vec<Vec<u32>> {
    instants
        .iter()
        .map(|instant| instant.iter().map(|id| id.0).collect())
        .collect()
}

/// Re-evaluates both formulas on a returned trace; `mismatch` only when the
/// verdicts differ (FR-091 Behavior, FR-091-AC-3).
fn confirm(
    mut report: SolverConformanceReport,
    original: &FormulaDocument,
    rewritten: &FormulaDocument,
    instants: &[Vec<PropositionId>],
    limits: RecheckLimits,
) -> SolverConformanceReport {
    let (Ok(original_formula), Ok(rewritten_formula)) = (original.validate(), rewritten.validate())
    else {
        return non_conclusive(report, SolverConformanceReason::RecheckError, None);
    };
    let original_verdict = verdict(original_formula, "original", instants, limits);
    let rewritten_verdict = verdict(rewritten_formula, "rewritten", instants, limits);
    let (original_value, rewritten_value) = match (original_verdict, rewritten_verdict) {
        (Verdict::Value(left), Verdict::Value(right)) => (left, right),
        (Verdict::Budget, _) | (_, Verdict::Budget) => {
            report.diagnostic_trace = Some(plain_trace(instants));
            return non_conclusive(report, SolverConformanceReason::RecheckLimit, None);
        }
        (Verdict::Error, _) | (_, Verdict::Error) => {
            report.diagnostic_trace = Some(plain_trace(instants));
            return non_conclusive(report, SolverConformanceReason::RecheckError, None);
        }
    };
    report.original_verdict = Some(original_value);
    report.rewritten_verdict = Some(rewritten_value);
    if original_value == rewritten_value {
        report.diagnostic_trace = Some(plain_trace(instants));
        return non_conclusive(
            report,
            SolverConformanceReason::CounterexampleNotConfirmed,
            None,
        );
    }
    report.status = crate::ConformanceStatus::Mismatch;
    report.counterexample = Some(plain_trace(instants));
    report
}

fn mirrored_reason(code: ReasonCode) -> SolverConformanceReason {
    match code {
        ReasonCode::EncodeRefused => SolverConformanceReason::EncodeRefused,
        ReasonCode::CapabilityUnsupported => SolverConformanceReason::CapabilityUnsupported,
        ReasonCode::ConflictLimit => SolverConformanceReason::ConflictLimit,
        ReasonCode::TimeLimit => SolverConformanceReason::TimeLimit,
        ReasonCode::MemoryLimit => SolverConformanceReason::MemoryLimit,
        ReasonCode::RecheckLimit => SolverConformanceReason::RecheckLimit,
        ReasonCode::InvalidModel => SolverConformanceReason::InvalidModel,
        ReasonCode::WitnessRejected => SolverConformanceReason::WitnessRejected,
        ReasonCode::RecheckError => SolverConformanceReason::RecheckError,
        ReasonCode::BackendFailure => SolverConformanceReason::BackendFailure,
    }
}

fn classify(
    mut report: SolverConformanceReport,
    original: &FormulaDocument,
    rewritten: &FormulaDocument,
    result: &ResultDocument,
    limits: RecheckLimits,
) -> SolverConformanceReport {
    report.solver_limits = Some(result.limits);
    match result.disposition {
        Disposition::Unsatisfiable => match result.basis {
            Some(basis) => {
                report.status = crate::ConformanceStatus::Equivalent;
                report.basis = Some(basis);
                report.solver = Some(result.solver.clone());
                report
            }
            None => non_conclusive(
                report,
                SolverConformanceReason::MalformedSolverResult,
                Some("unsatisfiable_without_basis".to_owned()),
            ),
        },
        Disposition::Satisfiable => match &result.witness {
            Some(Witness::Trace(trace)) => {
                report.solver = Some(result.solver.clone());
                confirm(report, original, rewritten, &trace.instants, limits)
            }
            Some(Witness::PositionHistory(_)) | None => non_conclusive(
                report,
                SolverConformanceReason::MalformedSolverResult,
                Some("satisfiable_without_trace".to_owned()),
            ),
        },
        Disposition::Inconclusive | Disposition::Refused | Disposition::Failed => {
            match &result.reason {
                Some(reason) => non_conclusive(
                    report,
                    mirrored_reason(reason.code),
                    Some(reason.detail.clone()),
                ),
                None => non_conclusive(
                    report,
                    SolverConformanceReason::MalformedSolverResult,
                    Some("non_answer_without_reason".to_owned()),
                ),
            }
        }
    }
}

/// Decides equivalence of a closed-trace pair with tl-solve over every trace
/// of length `horizon + 1`, with no proposition, horizon or trace-count
/// ceiling of its own (only tl-solve's encode and solve limits).
///
/// Implements: FR-091
pub fn check_equivalence_with_solver(
    original: &FormulaDocument,
    rewritten: &FormulaDocument,
    comparison_id: impl Into<String>,
    options: SolverConformanceOptions,
    backend: &mut dyn SatBackend,
) -> SolverConformanceReport {
    let prepared = prepare(original, rewritten, comparison_id.into(), &options);
    let (report, trace_length) = match prepared {
        Ok(prepared) => prepared,
        Err(report) => return report,
    };
    let Some(miter) = miter_document(original, rewritten) else {
        return non_conclusive(report, SolverConformanceReason::MiterRefused, None);
    };
    let Ok(formula) = miter.validate() else {
        return non_conclusive(report, SolverConformanceReason::MiterRefused, None);
    };
    let request = SatRequest {
        formula,
        formula_id: MITER_FORMULA_ID,
        frame: FrameRequest::Closed {
            trace_length: Some(trace_length),
        },
        encode_limits: options.encode_limits,
        solve_limits: options.solve_limits,
        recheck: options.recheck_limits,
    };
    let result = check_satisfiable(&request, backend);
    classify(report, original, rewritten, &result, options.recheck_limits)
}

/// Classifies an already obtained tl-solve result for this pair exactly as
/// [`check_equivalence_with_solver`] does after solving, including
/// tl-rewrite's own counterexample re-check.
///
/// This is the seam through which a seeded tl-solve fault (a result whose
/// trace does not separate the pair) is shown to become a typed failure and
/// never a mismatch (FR-091-AC-3).
///
/// Implements: FR-091
pub fn solver_conformance_from_result(
    original: &FormulaDocument,
    rewritten: &FormulaDocument,
    comparison_id: impl Into<String>,
    options: SolverConformanceOptions,
    result: &ResultDocument,
) -> SolverConformanceReport {
    match prepare(original, rewritten, comparison_id.into(), &options) {
        Ok((report, _)) => classify(report, original, rewritten, result, options.recheck_limits),
        Err(report) => report,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proposition(id: u32) -> Node {
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(id),
        })
    }

    // Trace: TC-976, FR-091-AC-2
    #[test]
    fn miter_shifts_the_rewritten_graph_and_negates_the_equivalence() {
        let original = FormulaDocument::new(
            SemanticProfile::ClosedTraceV1,
            NodeId(1),
            vec![
                proposition(0),
                Node::new(NodeKind::Not { operand: NodeId(0) }),
            ],
        )
        .unwrap();
        let rewritten =
            FormulaDocument::new(SemanticProfile::ClosedTraceV1, NodeId(0), vec![proposition(0)])
                .unwrap();
        let miter = miter_document(&original, &rewritten).unwrap();
        let kinds: Vec<NodeKind> = miter.nodes().iter().map(|node| node.kind).collect();
        assert_eq!(
            kinds,
            vec![
                proposition(0).kind,
                NodeKind::Not { operand: NodeId(0) },
                proposition(0).kind,
                NodeKind::Equivalent {
                    left: NodeId(1),
                    right: NodeId(2),
                },
                NodeKind::Not { operand: NodeId(3) },
            ]
        );
        assert_eq!(miter.root(), NodeId(4));
    }
}
