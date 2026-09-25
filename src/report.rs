//! Versioned rewrite attempt reports and bounded canonical admission.

use std::{fmt, io::Write};

use serde::{
    de::{DeserializeOwned, Error as _},
    Deserialize, Serialize,
};
use tl_syntax::{FormulaDocument, RequirementContextDocument, SignalCatalogDocument, SourceSpan};

/// Stable rewrite traversal strategy.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RewriteStrategy {
    /// Rebuild preceding operands first and apply the first catalog match.
    BottomUpFirstMatch,
}

/// Deterministic resource ceilings.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RewriteBudgets {
    /// Maximum complete rewrite passes.
    pub max_iterations: u32,
    /// Maximum nodes in any rebuilt candidate graph.
    pub max_nodes: u32,
    /// Maximum output-changing rule applications.
    pub max_rule_applications: u64,
    /// Maximum deterministic node visits and emissions.
    pub max_work_units: u64,
}

impl Default for RewriteBudgets {
    fn default() -> Self {
        Self {
            max_iterations: 32,
            max_nodes: 100_000,
            max_rule_applications: 100_000,
            max_work_units: 1_000_000,
        }
    }
}

/// Complete options participating in deterministic replay.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RewriteOptions {
    /// Traversal and priority policy.
    pub strategy: RewriteStrategy,
    /// Checked resource ceilings.
    pub budgets: RewriteBudgets,
}

impl Default for RewriteOptions {
    fn default() -> Self {
        Self {
            strategy: RewriteStrategy::BottomUpFirstMatch,
            budgets: RewriteBudgets::default(),
        }
    }
}

/// Budget responsible for a non-success outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetKind {
    /// No further complete pass was permitted.
    Iterations,
    /// A candidate would exceed the node ceiling.
    Nodes,
    /// Another output-changing application was required.
    RuleApplications,
    /// Another deterministic visit or emission was required.
    WorkUnits,
}

/// Overall rewrite disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RewriteStatus {
    /// Input was already a fixed point.
    Unchanged,
    /// A changed fixed point was reached.
    Normalized,
    /// A configured resource ceiling stopped the attempt.
    BudgetExhausted,
    /// A prior complete formula state reappeared.
    NonConvergent,
    /// An internal consistency or evaluator failure prevented completion.
    Failed,
    /// The owned input document failed structural validation.
    InvalidInput,
    /// No enabled v1 rule is approved for the input profile.
    UnsupportedProfile,
    /// A context-bound formula named a proposition absent from the supplied catalog.
    UnresolvedBinding,
}

/// Formula location at which a context-bound catalog binding was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindingLocus {
    /// The original request formula could not be bound before execution.
    Input,
    /// A successful rewritten formula could not be bound before it escaped.
    Output,
}

/// The first unresolved proposition in a contextual rewrite attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BindingFailure {
    /// Boundary at which the formula was checked.
    pub locus: BindingLocus,
    /// Stable proposition identity absent from the supplied catalog.
    pub proposition_id: u32,
}

/// One output-changing application in exact execution order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RewriteStep {
    /// Zero-based global application sequence.
    pub sequence: u64,
    /// Zero-based rewrite pass.
    pub pass: u32,
    /// Node identity in the pass input document.
    pub source_node: u32,
    /// Parser-independent input span, when present.
    pub source_span: Option<SourceSpan>,
    /// Stable catalog identity.
    pub rule_id: String,
    /// Rule semantic revision.
    pub rule_revision: u32,
    /// Digest of the remapped pre-application node.
    pub before_sha256: String,
    /// Digest of the replacement node or subtree root.
    pub after_sha256: String,
    /// Rolling digest over every preceding application record.
    pub intermediate_sha256: String,
}

/// Versioned attempt report; only success statuses carry `output`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RewriteReport {
    /// Wire schema identity.
    pub schema_version: String,
    /// Caller-provided formula identity.
    pub formula_id: String,
    /// Exact engine source identity supplied by the build or caller.
    pub engine_source_revision: String,
    /// Exact syntax dependency revision.
    pub syntax_revision: String,
    /// Catalog digest participating in replay.
    pub catalog_sha256: String,
    /// Canonical input document digest.
    pub input_sha256: String,
    /// Digest binding input, identity, catalog, strategy, budgets, and source revision.
    pub request_sha256: String,
    /// Complete supplied signal-catalog identity for contextual v2 reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_catalog_sha256: Option<String>,
    /// Exact caller context for contextual v2 reports; `Some(None)` encodes JSON null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirement_context: Option<Option<RequirementContextDocument>>,
    /// Contextual formula-binding refusal, when one stopped the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_failure: Option<BindingFailure>,
    /// Successful output digest, absent for non-success.
    pub output_sha256: Option<String>,
    /// Last complete state digest for diagnostics.
    pub partial_sha256: Option<String>,
    /// Exact profile copied from the input.
    pub semantic_profile: String,
    /// Replayable options.
    pub options: RewriteOptions,
    /// Attempt disposition.
    pub status: RewriteStatus,
    /// Responsible budget for exhaustion.
    pub exhausted_budget: Option<BudgetKind>,
    /// Human-readable failure detail, absent on success.
    pub detail: Option<String>,
    /// Number of complete passes produced.
    pub iterations: u32,
    /// Deterministic work consumed.
    pub work_units: u64,
    /// Output-changing applications consumed.
    pub rule_applications: u64,
    /// Ordered rule-application trace.
    pub steps: Vec<RewriteStep>,
    /// Successful fixed-point document only.
    pub output: Option<FormulaDocument>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RewriteReportV1Wire {
    schema_version: String,
    formula_id: String,
    engine_source_revision: String,
    syntax_revision: String,
    catalog_sha256: String,
    input_sha256: String,
    request_sha256: String,
    output_sha256: Option<String>,
    partial_sha256: Option<String>,
    semantic_profile: String,
    options: RewriteOptions,
    status: RewriteStatus,
    exhausted_budget: Option<BudgetKind>,
    detail: Option<String>,
    iterations: u32,
    work_units: u64,
    rule_applications: u64,
    steps: Vec<RewriteStep>,
    output: Option<FormulaDocument>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RewriteReportV2Wire {
    schema_version: String,
    formula_id: String,
    engine_source_revision: String,
    syntax_revision: String,
    catalog_sha256: String,
    input_sha256: String,
    request_sha256: String,
    signal_catalog_sha256: String,
    requirement_context: serde_json::Value,
    binding_failure: Option<BindingFailure>,
    output_sha256: Option<String>,
    partial_sha256: Option<String>,
    semantic_profile: String,
    options: RewriteOptions,
    status: RewriteStatus,
    exhausted_budget: Option<BudgetKind>,
    detail: Option<String>,
    iterations: u32,
    work_units: u64,
    rule_applications: u64,
    steps: Vec<RewriteStep>,
    output: Option<FormulaDocument>,
}

impl RewriteReport {
    fn from_v1(w: RewriteReportV1Wire) -> Self {
        Self {
            schema_version: w.schema_version,
            formula_id: w.formula_id,
            engine_source_revision: w.engine_source_revision,
            syntax_revision: w.syntax_revision,
            catalog_sha256: w.catalog_sha256,
            input_sha256: w.input_sha256,
            request_sha256: w.request_sha256,
            signal_catalog_sha256: None,
            requirement_context: None,
            binding_failure: None,
            output_sha256: w.output_sha256,
            partial_sha256: w.partial_sha256,
            semantic_profile: w.semantic_profile,
            options: w.options,
            status: w.status,
            exhausted_budget: w.exhausted_budget,
            detail: w.detail,
            iterations: w.iterations,
            work_units: w.work_units,
            rule_applications: w.rule_applications,
            steps: w.steps,
            output: w.output,
        }
    }
    fn from_v2(w: RewriteReportV2Wire) -> Result<Self, String> {
        let requirement_context = if w.requirement_context.is_null() {
            None
        } else {
            Some(serde_json::from_value(w.requirement_context).map_err(|error| error.to_string())?)
        };
        Ok(Self {
            schema_version: w.schema_version,
            formula_id: w.formula_id,
            engine_source_revision: w.engine_source_revision,
            syntax_revision: w.syntax_revision,
            catalog_sha256: w.catalog_sha256,
            input_sha256: w.input_sha256,
            request_sha256: w.request_sha256,
            signal_catalog_sha256: Some(w.signal_catalog_sha256),
            requirement_context: Some(requirement_context),
            binding_failure: w.binding_failure,
            output_sha256: w.output_sha256,
            partial_sha256: w.partial_sha256,
            semantic_profile: w.semantic_profile,
            options: w.options,
            status: w.status,
            exhausted_budget: w.exhausted_budget,
            detail: w.detail,
            iterations: w.iterations,
            work_units: w.work_units,
            rule_applications: w.rule_applications,
            steps: w.steps,
            output: w.output,
        })
    }
}

impl<'de> Deserialize<'de> for RewriteReport {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let version = value
            .get("schemaVersion")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| D::Error::custom("rewrite report requires schemaVersion"))?;
        match version {
            "tl-rewrite.report/v1" => serde_json::from_value::<RewriteReportV1Wire>(value)
                .map(Self::from_v1)
                .map_err(D::Error::custom),
            "tl-rewrite.report/v2" => serde_json::from_value::<RewriteReportV2Wire>(value)
                .and_then(|wire| Self::from_v2(wire).map_err(serde_json::Error::custom))
                .map_err(D::Error::custom),
            _ => Err(D::Error::custom("unsupported rewrite report schemaVersion")),
        }
    }
}

/// Owner maxima for rewrite report and replay document admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordLimits {
    /// Maximum exact JSON bytes.
    pub document_bytes: usize,
    /// Maximum object/array nesting.
    pub json_depth: usize,
    /// Maximum decoded bytes in one JSON string or object key.
    pub string_bytes: usize,
}

impl RecordLimits {
    /// Immutable owner ceilings. Callers may lower but never raise them.
    pub const OWNER_MAXIMA: Self = Self {
        document_bytes: 64 * 1024 * 1024,
        json_depth: 128,
        string_bytes: 64 * 1024,
    };

    /// Intersects caller limits with the immutable owner ceilings.
    #[must_use]
    pub const fn constrained(self) -> Self {
        Self {
            document_bytes: minimum(self.document_bytes, Self::OWNER_MAXIMA.document_bytes),
            json_depth: minimum(self.json_depth, Self::OWNER_MAXIMA.json_depth),
            string_bytes: minimum(self.string_bytes, Self::OWNER_MAXIMA.string_bytes),
        }
    }
}

impl Default for RecordLimits {
    fn default() -> Self {
        Self::OWNER_MAXIMA
    }
}

const fn minimum(left: usize, right: usize) -> usize {
    if left < right {
        left
    } else {
        right
    }
}

/// Stable class of strict record-reader refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RecordReadErrorCode {
    /// An owner or caller resource ceiling was exceeded.
    ResourceLimit,
    /// The bytes are not one supported closed JSON record.
    Malformed,
    /// The record is semantically readable but not in its one canonical encoding.
    NonCanonical,
    /// Re-execution did not reproduce the supplied report.
    ExpectedMismatch,
}

/// Typed fail-closed record admission error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordReadError {
    code: RecordReadErrorCode,
    resource: &'static str,
    actual: Option<usize>,
    limit: Option<usize>,
    detail: String,
}

impl RecordReadError {
    /// Stable refusal class.
    #[must_use]
    pub const fn code(&self) -> RecordReadErrorCode {
        self.code
    }

    /// Resource or semantic field responsible for refusal.
    #[must_use]
    pub const fn resource(&self) -> &'static str {
        self.resource
    }

    /// Observed population for a resource refusal.
    #[must_use]
    pub const fn actual(&self) -> Option<usize> {
        self.actual
    }

    /// Effective ceiling for a resource refusal.
    #[must_use]
    pub const fn limit(&self) -> Option<usize> {
        self.limit
    }
}

impl fmt::Display for RecordReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for RecordReadError {}

fn resource(resource: &'static str, actual: usize, limit: usize) -> RecordReadError {
    RecordReadError {
        code: RecordReadErrorCode::ResourceLimit,
        resource,
        actual: Some(actual),
        limit: Some(limit),
        detail: format!("{resource} population {actual} exceeds limit {limit}"),
    }
}

fn malformed(detail: impl Into<String>) -> RecordReadError {
    RecordReadError {
        code: RecordReadErrorCode::Malformed,
        resource: "json",
        actual: None,
        limit: None,
        detail: detail.into(),
    }
}

fn noncanonical() -> RecordReadError {
    RecordReadError {
        code: RecordReadErrorCode::NonCanonical,
        resource: "canonicalBytes",
        actual: None,
        limit: None,
        detail: "record bytes are not the exact canonical owner encoding".to_owned(),
    }
}

fn expected_mismatch() -> RecordReadError {
    RecordReadError {
        code: RecordReadErrorCode::ExpectedMismatch,
        resource: "replay",
        actual: None,
        limit: None,
        detail: "re-execution did not reproduce the supplied rewrite report".to_owned(),
    }
}

fn value_usage(value: &serde_json::Value) -> (usize, usize) {
    let mut maximum_depth = 0_usize;
    let mut maximum_string = 0_usize;
    let mut pending = vec![(value, 0_usize)];
    while let Some((value, parent_depth)) = pending.pop() {
        match value {
            serde_json::Value::Array(values) => {
                let depth = parent_depth.saturating_add(1);
                maximum_depth = maximum_depth.max(depth);
                pending.extend(values.iter().map(|value| (value, depth)));
            }
            serde_json::Value::Object(fields) => {
                let depth = parent_depth.saturating_add(1);
                maximum_depth = maximum_depth.max(depth);
                for (key, value) in fields {
                    maximum_string = maximum_string.max(key.len());
                    pending.push((value, depth));
                }
            }
            serde_json::Value::String(value) => {
                maximum_string = maximum_string.max(value.len());
            }
            serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {
            }
        }
    }
    (maximum_depth, maximum_string)
}

struct BoundedWriter {
    bytes: Vec<u8>,
    limit: usize,
    overflow: Option<usize>,
}

impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let length = self.bytes.len().saturating_add(bytes.len());
        if length > self.limit {
            self.overflow = Some(length);
            return Err(std::io::Error::other(
                "canonical record exceeds the effective byte limit",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) fn read_canonical<T>(bytes: &[u8], limits: RecordLimits) -> Result<T, RecordReadError>
where
    T: DeserializeOwned + Serialize,
{
    let limits = limits.constrained();
    if bytes.len() > limits.document_bytes {
        return Err(resource(
            "documentBytes",
            bytes.len(),
            limits.document_bytes,
        ));
    }
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| malformed(format!("record JSON is malformed: {error}")))?;
    let (depth, string_bytes) = value_usage(&value);
    if depth > limits.json_depth {
        return Err(resource("jsonDepth", depth, limits.json_depth));
    }
    if string_bytes > limits.string_bytes {
        return Err(resource("stringBytes", string_bytes, limits.string_bytes));
    }
    let document: T = serde_json::from_value(value)
        .map_err(|error| malformed(format!("record shape is invalid: {error}")))?;
    let mut writer = BoundedWriter {
        bytes: Vec::new(),
        limit: limits.document_bytes,
        overflow: None,
    };
    let encoded = serde_json::to_writer(&mut writer, &document);
    if let Some(actual) = writer.overflow {
        return Err(resource("documentBytes", actual, limits.document_bytes));
    }
    encoded
        .map_err(|error| malformed(format!("record cannot be serialized canonically: {error}")))?;
    if writer.bytes != bytes {
        return Err(noncanonical());
    }
    Ok(document)
}

impl RewriteReport {
    /// Reads one exact canonical report under caller-lowerable owner limits.
    ///
    /// Implements: FR-010-AC-5
    pub fn from_json_bytes(bytes: &[u8], limits: RecordLimits) -> Result<Self, RecordReadError> {
        read_canonical(bytes, limits)
    }
}

/// Strict-reads and independently re-executes one context-free rewrite report.
///
/// Implements: FR-010-AC-5
pub fn read(
    bytes: &[u8],
    input: &FormulaDocument,
    limits: RecordLimits,
) -> Result<RewriteReport, RecordReadError> {
    let report = RewriteReport::from_json_bytes(bytes, limits)?;
    if crate::replay::replay(input, &report).status != crate::replay::ReplayStatus::Verified {
        return Err(expected_mismatch());
    }
    Ok(report)
}

/// Strict-reads and independently re-executes one contextual rewrite report.
///
/// Implements: FR-010-AC-5
pub fn read_with_context(
    bytes: &[u8],
    input: &FormulaDocument,
    signal_catalog: &SignalCatalogDocument,
    requirement_context: Option<RequirementContextDocument>,
    limits: RecordLimits,
) -> Result<RewriteReport, RecordReadError> {
    let report = RewriteReport::from_json_bytes(bytes, limits)?;
    if crate::replay::replay_with_context(input, &report, signal_catalog, requirement_context)
        .status
        != crate::replay::ReplayStatus::Verified
    {
        return Err(expected_mismatch());
    }
    Ok(report)
}
