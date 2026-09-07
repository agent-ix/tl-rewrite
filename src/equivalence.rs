use std::collections::BTreeSet;

use serde::{de::Error as _, Deserialize, Serialize};
use tl_mltl::{analyze_horizon, evaluate_closed, EvaluationLimits, TruthValue};
use tl_syntax::{
    Formula, FormulaDocument, NodeKind, PropositionId, RequirementContextDocument, SemanticProfile,
    SignalCatalogDocument,
};

use crate::{
    catalog,
    hash::sha256_json,
    rewrite::{binding_check, BindingCheck},
    TL_MLTL_REVISION, TL_SYNTAX_REVISION, WEST_REVISION,
};

const MAX_MATERIALIZED_INSTANTS: u64 = 100_000;

/// Explicit exhaustive-domain ceilings.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConformanceOptions {
    /// Maximum distinct proposition identities.
    pub max_propositions: u32,
    /// Maximum checked lookahead.
    pub max_horizon: u64,
    /// Maximum traces in the complete valuation domain.
    pub max_traces: u64,
}

impl Default for ConformanceOptions {
    fn default() -> Self {
        Self {
            max_propositions: 4,
            max_horizon: 8,
            max_traces: 1_000_000,
        }
    }
}

/// Conformance disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConformanceStatus {
    /// Every trace in the declared domain agreed.
    Equivalent,
    /// A deterministic counterexample was observed.
    Mismatch,
    /// The requested comparison could not be completed conclusively.
    NonConclusive,
}

/// Typed reason for a non-conclusive comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConformanceReason {
    /// Either document was structurally invalid.
    InvalidInput,
    /// Profiles differed or were not closed-trace v1.
    UnsupportedProfile,
    /// The proposition population exceeded its ceiling.
    PropositionLimit,
    /// The checked horizon exceeded its ceiling.
    HorizonLimit,
    /// Complete enumeration exceeded its trace ceiling or integer domain.
    TraceDomainLimit,
    /// The pinned reference evaluator returned an error.
    EvaluatorError,
    /// The original contextual formula names a proposition absent from the catalog.
    OriginalBinding,
    /// The rewritten contextual formula names a proposition absent from the catalog.
    RewrittenBinding,
}

/// Versioned exhaustive bounded comparison report.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConformanceReport {
    /// Wire schema identity.
    pub schema_version: String,
    /// Caller-provided comparison identity.
    pub comparison_id: String,
    /// Canonical original formula digest.
    pub original_sha256: String,
    /// Canonical rewritten formula digest.
    pub rewritten_sha256: String,
    /// Exact semantic profile when supported.
    pub semantic_profile: String,
    /// Exact syntax dependency revision.
    pub syntax_revision: String,
    /// Exact evaluator dependency revision.
    pub evaluator_revision: String,
    /// Exact WEST corpus source revision.
    pub west_revision: String,
    /// Stable catalog identity.
    pub catalog_version: String,
    /// Exact ordered catalog digest.
    pub catalog_sha256: String,
    /// Complete supplied signal-catalog identity for contextual v2 reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_catalog_sha256: Option<String>,
    /// Exact caller context for contextual v2 reports; `Some(None)` encodes JSON null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirement_context: Option<Option<RequirementContextDocument>>,
    /// Contextual request identity binding both formulas and the supplied documents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_sha256: Option<String>,
    /// Contextual binding refusal locus and proposition identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_failure: Option<crate::BindingFailure>,
    /// Options defining the conclusive boundary.
    pub options: ConformanceOptions,
    /// Sorted proposition population.
    pub proposition_ids: Vec<u32>,
    /// Maximum formula lookahead.
    pub horizon: Option<u64>,
    /// Horizon-complete trace length.
    pub trace_length: Option<u64>,
    /// Cardinality of the complete domain.
    pub total_traces: Option<u64>,
    /// Number of traces evaluated before disposition.
    pub traces_checked: u64,
    /// Result classification.
    pub status: ConformanceStatus,
    /// Reason when non-conclusive.
    pub reason: Option<ConformanceReason>,
    /// First trace in stable enumeration order that disagreed.
    pub counterexample: Option<Vec<Vec<u32>>>,
    /// Original verdict on the counterexample.
    pub original_verdict: Option<TruthValue>,
    /// Rewritten verdict on the counterexample.
    pub rewritten_verdict: Option<TruthValue>,
    /// Qualification boundary.
    pub limitation: String,
}

/// Closed legacy wire shape. Keeping it separate makes contextual-field
/// smuggling into v1 records a decoding error rather than an ignored input.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConformanceReportV1Wire {
    schema_version: String,
    comparison_id: String,
    original_sha256: String,
    rewritten_sha256: String,
    semantic_profile: String,
    syntax_revision: String,
    evaluator_revision: String,
    west_revision: String,
    catalog_version: String,
    catalog_sha256: String,
    options: ConformanceOptions,
    proposition_ids: Vec<u32>,
    horizon: Option<u64>,
    trace_length: Option<u64>,
    total_traces: Option<u64>,
    traces_checked: u64,
    status: ConformanceStatus,
    reason: Option<ConformanceReason>,
    counterexample: Option<Vec<Vec<u32>>>,
    original_verdict: Option<TruthValue>,
    rewritten_verdict: Option<TruthValue>,
    limitation: String,
}

/// Closed contextual wire shape. `requirement_context` deliberately remains a
/// raw value until after shape validation so a missing field is distinct from
/// its required explicit-null spelling.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConformanceReportV2Wire {
    schema_version: String,
    comparison_id: String,
    original_sha256: String,
    rewritten_sha256: String,
    semantic_profile: String,
    syntax_revision: String,
    evaluator_revision: String,
    west_revision: String,
    catalog_version: String,
    catalog_sha256: String,
    signal_catalog_sha256: String,
    requirement_context: serde_json::Value,
    request_sha256: String,
    binding_failure: Option<crate::BindingFailure>,
    options: ConformanceOptions,
    proposition_ids: Vec<u32>,
    horizon: Option<u64>,
    trace_length: Option<u64>,
    total_traces: Option<u64>,
    traces_checked: u64,
    status: ConformanceStatus,
    reason: Option<ConformanceReason>,
    counterexample: Option<Vec<Vec<u32>>>,
    original_verdict: Option<TruthValue>,
    rewritten_verdict: Option<TruthValue>,
    limitation: String,
}

impl ConformanceReport {
    fn from_v1(wire: ConformanceReportV1Wire) -> Self {
        Self {
            schema_version: wire.schema_version,
            comparison_id: wire.comparison_id,
            original_sha256: wire.original_sha256,
            rewritten_sha256: wire.rewritten_sha256,
            semantic_profile: wire.semantic_profile,
            syntax_revision: wire.syntax_revision,
            evaluator_revision: wire.evaluator_revision,
            west_revision: wire.west_revision,
            catalog_version: wire.catalog_version,
            catalog_sha256: wire.catalog_sha256,
            signal_catalog_sha256: None,
            requirement_context: None,
            request_sha256: None,
            binding_failure: None,
            options: wire.options,
            proposition_ids: wire.proposition_ids,
            horizon: wire.horizon,
            trace_length: wire.trace_length,
            total_traces: wire.total_traces,
            traces_checked: wire.traces_checked,
            status: wire.status,
            reason: wire.reason,
            counterexample: wire.counterexample,
            original_verdict: wire.original_verdict,
            rewritten_verdict: wire.rewritten_verdict,
            limitation: wire.limitation,
        }
    }

    fn from_v2(wire: ConformanceReportV2Wire) -> Result<Self, serde_json::Error> {
        let requirement_context = if wire.requirement_context.is_null() {
            None
        } else {
            Some(serde_json::from_value(wire.requirement_context)?)
        };
        Ok(Self {
            schema_version: wire.schema_version,
            comparison_id: wire.comparison_id,
            original_sha256: wire.original_sha256,
            rewritten_sha256: wire.rewritten_sha256,
            semantic_profile: wire.semantic_profile,
            syntax_revision: wire.syntax_revision,
            evaluator_revision: wire.evaluator_revision,
            west_revision: wire.west_revision,
            catalog_version: wire.catalog_version,
            catalog_sha256: wire.catalog_sha256,
            signal_catalog_sha256: Some(wire.signal_catalog_sha256),
            requirement_context: Some(requirement_context),
            request_sha256: Some(wire.request_sha256),
            binding_failure: wire.binding_failure,
            options: wire.options,
            proposition_ids: wire.proposition_ids,
            horizon: wire.horizon,
            trace_length: wire.trace_length,
            total_traces: wire.total_traces,
            traces_checked: wire.traces_checked,
            status: wire.status,
            reason: wire.reason,
            counterexample: wire.counterexample,
            original_verdict: wire.original_verdict,
            rewritten_verdict: wire.rewritten_verdict,
            limitation: wire.limitation,
        })
    }
}

impl<'de> Deserialize<'de> for ConformanceReport {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let version = value
            .get("schemaVersion")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| D::Error::custom("conformance report requires schemaVersion"))?;
        match version {
            "tl-rewrite.conformance/v1" => serde_json::from_value::<ConformanceReportV1Wire>(value)
                .map(Self::from_v1)
                .map_err(D::Error::custom),
            "tl-rewrite.conformance/v2" => serde_json::from_value::<ConformanceReportV2Wire>(value)
                .and_then(|wire| Self::from_v2(wire).map_err(serde_json::Error::custom))
                .map_err(D::Error::custom),
            _ => Err(D::Error::custom(
                "unsupported conformance report schemaVersion",
            )),
        }
    }
}

fn propositions(formula: Formula<'_>, destination: &mut BTreeSet<u32>) {
    for node in formula.nodes() {
        if let NodeKind::Proposition { proposition } = node.kind {
            destination.insert(proposition.0);
        }
    }
}

fn trace(mask: u64, trace_length: usize, propositions: &[u32]) -> Vec<Vec<PropositionId>> {
    (0..trace_length)
        .map(|time| {
            propositions
                .iter()
                .enumerate()
                .filter_map(|(offset, proposition)| {
                    let bit = time * propositions.len() + offset;
                    ((mask >> bit) & 1 == 1).then_some(PropositionId(*proposition))
                })
                .collect()
        })
        .collect()
}

fn report_base(
    original: &FormulaDocument,
    rewritten: &FormulaDocument,
    comparison_id: String,
    options: ConformanceOptions,
) -> ConformanceReport {
    let rule_catalog = catalog();
    ConformanceReport {
        schema_version: "tl-rewrite.conformance/v1".to_owned(),
        comparison_id,
        original_sha256: sha256_json(original),
        rewritten_sha256: sha256_json(rewritten),
        semantic_profile: original.semantic_profile().as_str().to_owned(),
        syntax_revision: TL_SYNTAX_REVISION.to_owned(),
        evaluator_revision: TL_MLTL_REVISION.to_owned(),
        west_revision: WEST_REVISION.to_owned(),
        catalog_version: rule_catalog.catalog_version,
        catalog_sha256: rule_catalog.catalog_sha256,
        signal_catalog_sha256: None,
        requirement_context: None,
        request_sha256: None,
        binding_failure: None,
        options,
        proposition_ids: Vec::new(),
        horizon: None,
        trace_length: None,
        total_traces: None,
        traces_checked: 0,
        status: ConformanceStatus::NonConclusive,
        reason: None,
        counterexample: None,
        original_verdict: None,
        rewritten_verdict: None,
        limitation: "bounded agreement enumerates only trace_length == horizon + 1 under the declared closed-trace profile; shorter traces are outside the reported domain and may differ from textbook finite-trace MLTL; this does not prove arbitrary rewrite schemas or qualify a consuming tool".to_owned(),
    }
}

fn non_conclusive(mut report: ConformanceReport, reason: ConformanceReason) -> ConformanceReport {
    report.status = ConformanceStatus::NonConclusive;
    report.reason = Some(reason);
    report
}

/// Exhaustively compares a closed-trace formula pair over its complete bounded horizon.
///
/// Implements: FR-004
pub fn check_equivalence(
    original: &FormulaDocument,
    rewritten: &FormulaDocument,
    comparison_id: impl Into<String>,
    options: ConformanceOptions,
) -> ConformanceReport {
    let mut report = report_base(original, rewritten, comparison_id.into(), options);
    let Ok(original_formula) = original.validate() else {
        return non_conclusive(report, ConformanceReason::InvalidInput);
    };
    let Ok(rewritten_formula) = rewritten.validate() else {
        return non_conclusive(report, ConformanceReason::InvalidInput);
    };
    if original.semantic_profile() != rewritten.semantic_profile()
        || original.semantic_profile() != SemanticProfile::ClosedTraceV1
    {
        return non_conclusive(report, ConformanceReason::UnsupportedProfile);
    }

    let mut proposition_ids = BTreeSet::new();
    propositions(original_formula, &mut proposition_ids);
    propositions(rewritten_formula, &mut proposition_ids);
    report.proposition_ids = proposition_ids.into_iter().collect();
    if report.proposition_ids.len() > options.max_propositions as usize {
        return non_conclusive(report, ConformanceReason::PropositionLimit);
    }

    let original_horizon = match analyze_horizon(original_formula, "original") {
        Ok(value) => value.lookahead,
        Err(_) => return non_conclusive(report, ConformanceReason::EvaluatorError),
    };
    let rewritten_horizon = match analyze_horizon(rewritten_formula, "rewritten") {
        Ok(value) => value.lookahead,
        Err(_) => return non_conclusive(report, ConformanceReason::EvaluatorError),
    };
    let horizon = original_horizon.max(rewritten_horizon);
    report.horizon = Some(horizon);
    if horizon > options.max_horizon {
        return non_conclusive(report, ConformanceReason::HorizonLimit);
    }
    let Some(trace_length) = horizon.checked_add(1) else {
        return non_conclusive(report, ConformanceReason::TraceDomainLimit);
    };
    report.trace_length = Some(trace_length);
    if trace_length > MAX_MATERIALIZED_INSTANTS {
        return non_conclusive(report, ConformanceReason::TraceDomainLimit);
    }
    let Some(bits) = trace_length.checked_mul(report.proposition_ids.len() as u64) else {
        return non_conclusive(report, ConformanceReason::TraceDomainLimit);
    };
    let Some(total_traces) = u32::try_from(bits)
        .ok()
        .and_then(|shift| 1_u64.checked_shl(shift))
    else {
        return non_conclusive(report, ConformanceReason::TraceDomainLimit);
    };
    report.total_traces = Some(total_traces);
    if total_traces > options.max_traces {
        return non_conclusive(report, ConformanceReason::TraceDomainLimit);
    }
    let Ok(trace_length_usize) = usize::try_from(trace_length) else {
        return non_conclusive(report, ConformanceReason::TraceDomainLimit);
    };

    for mask in 0..total_traces {
        let candidate = trace(mask, trace_length_usize, &report.proposition_ids);
        let original_result = evaluate_closed(
            original_formula,
            "original",
            &candidate,
            format!("trace-{mask}"),
            EvaluationLimits::default(),
        );
        let rewritten_result = evaluate_closed(
            rewritten_formula,
            "rewritten",
            &candidate,
            format!("trace-{mask}"),
            EvaluationLimits::default(),
        );
        let (Ok(original_result), Ok(rewritten_result)) = (original_result, rewritten_result)
        else {
            report.traces_checked = mask;
            return non_conclusive(report, ConformanceReason::EvaluatorError);
        };
        report.traces_checked = mask + 1;
        if original_result.verdict != rewritten_result.verdict {
            report.status = ConformanceStatus::Mismatch;
            report.counterexample = Some(
                candidate
                    .iter()
                    .map(|instant| instant.iter().map(|id| id.0).collect())
                    .collect(),
            );
            report.original_verdict = Some(original_result.verdict);
            report.rewritten_verdict = Some(rewritten_result.verdict);
            return report;
        }
    }
    report.status = ConformanceStatus::Equivalent;
    report
}

/// Exhaustively compares a context-bound pair after binding both formulas to one
/// shared catalog. Binding refusal happens before horizon or trace enumeration.
///
/// Implements: FR-007-AC-4
pub fn check_equivalence_with_context(
    original: &FormulaDocument,
    rewritten: &FormulaDocument,
    comparison_id: impl Into<String>,
    options: ConformanceOptions,
    signal_catalog: &SignalCatalogDocument,
    requirement_context: Option<RequirementContextDocument>,
) -> ConformanceReport {
    let comparison_id = comparison_id.into();
    let mut report = report_base(original, rewritten, comparison_id.clone(), options);
    let rule_catalog = catalog();
    report.schema_version = "tl-rewrite.conformance/v2".to_owned();
    report.signal_catalog_sha256 = Some(sha256_json(signal_catalog));
    report.request_sha256 = Some(sha256_json(&(
        "tl-rewrite.contextual-conformance-request/v2",
        original,
        rewritten,
        &comparison_id,
        options,
        signal_catalog,
        &requirement_context,
        TL_SYNTAX_REVISION,
        TL_MLTL_REVISION,
        WEST_REVISION,
        &rule_catalog.catalog_version,
        &rule_catalog.catalog_sha256,
    )));
    report.requirement_context = Some(requirement_context);
    let Ok(catalog) = signal_catalog.validate() else {
        return non_conclusive(report, ConformanceReason::InvalidInput);
    };
    let Ok(original_formula) = original.validate() else {
        return non_conclusive(report, ConformanceReason::InvalidInput);
    };
    match binding_check(catalog, original_formula, crate::BindingLocus::Input) {
        BindingCheck::Bound => {}
        BindingCheck::Missing(binding_failure) => {
            report.binding_failure = Some(binding_failure);
            return non_conclusive(report, ConformanceReason::OriginalBinding);
        }
        BindingCheck::Refused(_) => {
            return non_conclusive(report, ConformanceReason::OriginalBinding);
        }
    }
    let Ok(rewritten_formula) = rewritten.validate() else {
        return non_conclusive(report, ConformanceReason::InvalidInput);
    };
    match binding_check(catalog, rewritten_formula, crate::BindingLocus::Output) {
        BindingCheck::Bound => {}
        BindingCheck::Missing(binding_failure) => {
            report.binding_failure = Some(binding_failure);
            return non_conclusive(report, ConformanceReason::RewrittenBinding);
        }
        BindingCheck::Refused(_) => {
            return non_conclusive(report, ConformanceReason::RewrittenBinding);
        }
    }
    let mut observed = check_equivalence(original, rewritten, comparison_id, options);
    observed.schema_version = report.schema_version;
    observed.signal_catalog_sha256 = report.signal_catalog_sha256;
    observed.requirement_context = report.requirement_context;
    observed.request_sha256 = report.request_sha256;
    observed
}
