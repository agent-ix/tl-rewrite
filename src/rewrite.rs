use std::collections::{BTreeMap, BTreeSet};

use serde::{de::Error as _, Deserialize, Serialize};
use tl_syntax::{
    Formula, FormulaBindingError, FormulaDocument, Node, NodeId, NodeKind,
    RequirementContextDocument, SemanticProfile, SignalCatalog, SignalCatalogDocument, SourceSpan,
};

use crate::{catalog, hash::sha256_bytes, hash::sha256_json, TL_SYNTAX_REVISION};

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

/// Closed local interpretation of one shared-catalog binding attempt.
///
/// `FormulaBindingError` is non-exhaustive upstream. New refusal variants must
/// remain non-success here instead of silently becoming an accepted binding.
pub(crate) enum BindingCheck {
    Bound,
    Missing(BindingFailure),
    Refused(String),
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

/// Replay comparison status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayStatus {
    /// Re-execution reproduced the complete report.
    Verified,
    /// Re-execution differed in at least one field.
    Mismatch,
}

/// Versioned replay result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplayReport {
    /// Wire schema identity.
    pub schema_version: String,
    /// Comparison result.
    pub status: ReplayStatus,
    /// Digest of the supplied report.
    pub expected_report_sha256: String,
    /// Digest of the freshly observed report.
    pub observed_report_sha256: String,
    /// Complete supplied signal-catalog identity for contextual v2 replay.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_catalog_sha256: Option<String>,
    /// Exact caller context for contextual v2 replay; `Some(None)` encodes JSON null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirement_context: Option<Option<RequirementContextDocument>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReplayReportV1Wire {
    schema_version: String,
    status: ReplayStatus,
    expected_report_sha256: String,
    observed_report_sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReplayReportV2Wire {
    schema_version: String,
    status: ReplayStatus,
    expected_report_sha256: String,
    observed_report_sha256: String,
    signal_catalog_sha256: String,
    requirement_context: serde_json::Value,
}

impl<'de> Deserialize<'de> for ReplayReport {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let version = value
            .get("schemaVersion")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| D::Error::custom("replay report requires schemaVersion"))?;
        match version {
            "tl-rewrite.replay/v1" => serde_json::from_value::<ReplayReportV1Wire>(value)
                .map(|wire| Self {
                    schema_version: wire.schema_version,
                    status: wire.status,
                    expected_report_sha256: wire.expected_report_sha256,
                    observed_report_sha256: wire.observed_report_sha256,
                    signal_catalog_sha256: None,
                    requirement_context: None,
                })
                .map_err(D::Error::custom),
            "tl-rewrite.replay/v2" => serde_json::from_value::<ReplayReportV2Wire>(value)
                .and_then(|wire| {
                    let context = if wire.requirement_context.is_null() {
                        None
                    } else {
                        Some(serde_json::from_value(wire.requirement_context)?)
                    };
                    Ok(Self {
                        schema_version: wire.schema_version,
                        status: wire.status,
                        expected_report_sha256: wire.expected_report_sha256,
                        observed_report_sha256: wire.observed_report_sha256,
                        signal_catalog_sha256: Some(wire.signal_catalog_sha256),
                        requirement_context: Some(context),
                    })
                })
                .map_err(D::Error::custom),
            _ => Err(D::Error::custom("unsupported replay report schemaVersion")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Abort {
    Budget(BudgetKind),
}

struct PassState {
    nodes: Vec<Node>,
    interner: BTreeMap<NodeKind, NodeId>,
    work_units: u64,
    budgets: RewriteBudgets,
}

impl PassState {
    fn work(&mut self) -> Result<(), Abort> {
        if self.work_units >= self.budgets.max_work_units {
            return Err(Abort::Budget(BudgetKind::WorkUnits));
        }
        self.work_units = self
            .work_units
            .checked_add(1)
            .ok_or(Abort::Budget(BudgetKind::WorkUnits))?;
        Ok(())
    }

    fn emit(&mut self, kind: NodeKind, span: Option<SourceSpan>) -> Result<NodeId, Abort> {
        self.work()?;
        if let Some(id) = self.interner.get(&kind) {
            return Ok(*id);
        }
        if self.nodes.len() >= u32::MAX as usize {
            return Err(Abort::Budget(BudgetKind::Nodes));
        }
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(Node { kind, span });
        self.interner.insert(kind, id);
        Ok(id)
    }

    fn kind(&self, id: NodeId) -> NodeKind {
        self.nodes[id.0 as usize].kind
    }
}

fn operands(kind: NodeKind) -> [Option<NodeId>; 2] {
    match kind {
        NodeKind::Not { operand }
        | NodeKind::Future { operand, .. }
        | NodeKind::Globally { operand, .. } => [Some(operand), None],
        NodeKind::And { left, right }
        | NodeKind::Or { left, right }
        | NodeKind::Implies { left, right }
        | NodeKind::Equivalent { left, right }
        | NodeKind::Until { left, right, .. }
        | NodeKind::Release { left, right, .. } => [Some(left), Some(right)],
        NodeKind::False | NodeKind::True | NodeKind::Proposition { .. } => [None, None],
    }
}

fn retained_operands(rule_id: &str, input: NodeKind) -> [Option<NodeId>; 2] {
    let [left, right] = operands(input);
    match rule_id {
        "bool.and.false-left" | "bool.or.true-left" | "bool.implies.false-left" => [left, None],
        "bool.and.false-right" | "bool.or.true-right" | "bool.implies.true-right" => [right, None],
        "bool.and.true-left"
        | "bool.and.true-right"
        | "bool.and.idempotent"
        | "bool.or.false-left"
        | "bool.or.false-right"
        | "bool.or.idempotent"
        | "bool.implies.true-left"
        | "bool.implies.false-right"
        | "bool.implies.reflexive"
        | "bool.implies.eliminate"
        | "bool.equivalent.true-left"
        | "bool.equivalent.true-right"
        | "bool.equivalent.false-left"
        | "bool.equivalent.false-right"
        | "bool.equivalent.reflexive"
        | "neg.future.dual"
        | "neg.globally.dual"
        | "neg.until.dual"
        | "neg.release.dual"
        | "temporal.until.true-left"
        | "temporal.release.false-left" => [left, right],
        "bool.not.false"
        | "bool.not.true"
        | "bool.not.double"
        | "temporal.future.singleton"
        | "temporal.globally.singleton"
        | "temporal.future.false"
        | "temporal.future.true"
        | "temporal.globally.false"
        | "temporal.globally.true" => [left, None],
        "temporal.until.singleton" | "temporal.release.singleton" => [right, None],
        _ => [left, right],
    }
}

fn reachable_nodes(root: NodeId, nodes: &[Node]) -> Vec<bool> {
    let mut reachable = vec![false; nodes.len()];
    let mut pending = vec![root];
    while let Some(id) = pending.pop() {
        let index = id.0 as usize;
        if index >= nodes.len() || reachable[index] {
            continue;
        }
        reachable[index] = true;
        pending.extend(operands(nodes[index].kind).into_iter().flatten());
    }
    reachable
}

fn compact_document(input: &FormulaDocument) -> Option<FormulaDocument> {
    let reachable = reachable_nodes(input.root(), input.nodes());
    let mut mapped = vec![NodeId(0); input.nodes().len()];
    let mut nodes: Vec<Node> = Vec::new();
    let mut interner = BTreeMap::new();
    for (index, node) in input.nodes().iter().enumerate() {
        if !reachable[index] {
            continue;
        }
        let kind = remap(node.kind, &mapped);
        let id = if let Some(existing) = interner.get(&kind) {
            *existing
        } else {
            let id = NodeId(nodes.len() as u32);
            nodes.push(Node {
                kind,
                span: node.span,
            });
            interner.insert(kind, id);
            id
        };
        mapped[index] = id;
    }
    FormulaDocument::new(
        input.semantic_profile(),
        mapped[input.root().0 as usize],
        nodes,
    )
    .ok()
}

fn remap(kind: NodeKind, mapped: &[NodeId]) -> NodeKind {
    let id = |value: NodeId| mapped[value.0 as usize];
    match kind {
        NodeKind::False => NodeKind::False,
        NodeKind::True => NodeKind::True,
        NodeKind::Proposition { proposition } => NodeKind::Proposition { proposition },
        NodeKind::Not { operand } => NodeKind::Not {
            operand: id(operand),
        },
        NodeKind::And { left, right } => NodeKind::And {
            left: id(left),
            right: id(right),
        },
        NodeKind::Or { left, right } => NodeKind::Or {
            left: id(left),
            right: id(right),
        },
        NodeKind::Implies { left, right } => NodeKind::Implies {
            left: id(left),
            right: id(right),
        },
        NodeKind::Equivalent { left, right } => NodeKind::Equivalent {
            left: id(left),
            right: id(right),
        },
        NodeKind::Future { interval, operand } => NodeKind::Future {
            interval,
            operand: id(operand),
        },
        NodeKind::Globally { interval, operand } => NodeKind::Globally {
            interval,
            operand: id(operand),
        },
        NodeKind::Until {
            interval,
            left,
            right,
        } => NodeKind::Until {
            interval,
            left: id(left),
            right: id(right),
        },
        NodeKind::Release {
            interval,
            left,
            right,
        } => NodeKind::Release {
            interval,
            left: id(left),
            right: id(right),
        },
    }
}

fn is_false(state: &PassState, id: NodeId) -> bool {
    state.kind(id) == NodeKind::False
}

fn is_true(state: &PassState, id: NodeId) -> bool {
    state.kind(id) == NodeKind::True
}

fn applied(
    rule: &'static str,
    replacement: NodeId,
) -> Result<Option<(&'static str, u32, NodeId)>, Abort> {
    Ok(Some((rule, 1, replacement)))
}

fn applied_kind(
    state: &mut PassState,
    rule: &'static str,
    kind: NodeKind,
    span: Option<SourceSpan>,
) -> Result<Option<(&'static str, u32, NodeId)>, Abort> {
    let replacement = state.emit(kind, span)?;
    Ok(Some((rule, 1, replacement)))
}

fn apply_not(
    state: &mut PassState,
    operand: NodeId,
    span: Option<SourceSpan>,
) -> Result<Option<(&'static str, u32, NodeId)>, Abort> {
    match state.kind(operand) {
        NodeKind::False => applied_kind(state, "bool.not.false", NodeKind::True, span),
        NodeKind::True => applied_kind(state, "bool.not.true", NodeKind::False, span),
        NodeKind::Not { operand } => applied("bool.not.double", operand),
        NodeKind::Future { interval, operand } => {
            let negated = state.emit(NodeKind::Not { operand }, None)?;
            let output = state.emit(
                NodeKind::Globally {
                    interval,
                    operand: negated,
                },
                span,
            )?;
            Ok(Some(("neg.future.dual", 1, output)))
        }
        NodeKind::Globally { interval, operand } => {
            let negated = state.emit(NodeKind::Not { operand }, None)?;
            let output = state.emit(
                NodeKind::Future {
                    interval,
                    operand: negated,
                },
                span,
            )?;
            Ok(Some(("neg.globally.dual", 1, output)))
        }
        NodeKind::Until {
            interval,
            left,
            right,
        } => {
            let left = state.emit(NodeKind::Not { operand: left }, None)?;
            let right = state.emit(NodeKind::Not { operand: right }, None)?;
            let output = state.emit(
                NodeKind::Release {
                    interval,
                    left,
                    right,
                },
                span,
            )?;
            Ok(Some(("neg.until.dual", 1, output)))
        }
        NodeKind::Release {
            interval,
            left,
            right,
        } => {
            let left = state.emit(NodeKind::Not { operand: left }, None)?;
            let right = state.emit(NodeKind::Not { operand: right }, None)?;
            let output = state.emit(
                NodeKind::Until {
                    interval,
                    left,
                    right,
                },
                span,
            )?;
            Ok(Some(("neg.release.dual", 1, output)))
        }
        _ => Ok(None),
    }
}

fn apply_first(
    state: &mut PassState,
    kind: NodeKind,
    span: Option<SourceSpan>,
) -> Result<Option<(&'static str, u32, NodeId)>, Abort> {
    match kind {
        NodeKind::Not { operand } => apply_not(state, operand, span),
        NodeKind::And { left, right: _ } if is_false(state, left) => {
            applied_kind(state, "bool.and.false-left", NodeKind::False, span)
        }
        NodeKind::And { left: _, right } if is_false(state, right) => {
            applied_kind(state, "bool.and.false-right", NodeKind::False, span)
        }
        NodeKind::And { left, right } if is_true(state, left) => {
            applied("bool.and.true-left", right)
        }
        NodeKind::And { left, right } if is_true(state, right) => {
            applied("bool.and.true-right", left)
        }
        NodeKind::And { left, right } if left == right => applied("bool.and.idempotent", left),
        NodeKind::Or { left, right: _ } if is_true(state, left) => {
            applied_kind(state, "bool.or.true-left", NodeKind::True, span)
        }
        NodeKind::Or { left: _, right } if is_true(state, right) => {
            applied_kind(state, "bool.or.true-right", NodeKind::True, span)
        }
        NodeKind::Or { left, right } if is_false(state, left) => {
            applied("bool.or.false-left", right)
        }
        NodeKind::Or { left, right } if is_false(state, right) => {
            applied("bool.or.false-right", left)
        }
        NodeKind::Or { left, right } if left == right => applied("bool.or.idempotent", left),
        NodeKind::Implies { left, right: _ } if is_false(state, left) => {
            applied_kind(state, "bool.implies.false-left", NodeKind::True, span)
        }
        NodeKind::Implies { left, right } if is_true(state, left) => {
            applied("bool.implies.true-left", right)
        }
        NodeKind::Implies { left: _, right } if is_true(state, right) => {
            applied_kind(state, "bool.implies.true-right", NodeKind::True, span)
        }
        NodeKind::Implies { left, right } if is_false(state, right) => applied_kind(
            state,
            "bool.implies.false-right",
            NodeKind::Not { operand: left },
            span,
        ),
        NodeKind::Implies { left, right } if left == right => {
            applied_kind(state, "bool.implies.reflexive", NodeKind::True, span)
        }
        NodeKind::Implies { left, right } => {
            let negated = state.emit(NodeKind::Not { operand: left }, None)?;
            let output = state.emit(
                NodeKind::Or {
                    left: negated,
                    right,
                },
                span,
            )?;
            Ok(Some(("bool.implies.eliminate", 1, output)))
        }
        NodeKind::Equivalent { left, right } if left == right => {
            applied_kind(state, "bool.equivalent.reflexive", NodeKind::True, span)
        }
        NodeKind::Equivalent { left, right } if is_true(state, left) => {
            applied("bool.equivalent.true-left", right)
        }
        NodeKind::Equivalent { left, right } if is_true(state, right) => {
            applied("bool.equivalent.true-right", left)
        }
        NodeKind::Equivalent { left, right } if is_false(state, left) => applied_kind(
            state,
            "bool.equivalent.false-left",
            NodeKind::Not { operand: right },
            span,
        ),
        NodeKind::Equivalent { left, right } if is_false(state, right) => applied_kind(
            state,
            "bool.equivalent.false-right",
            NodeKind::Not { operand: left },
            span,
        ),
        NodeKind::Future { interval, operand } if interval.start() == 0 && interval.end() == 0 => {
            applied("temporal.future.singleton", operand)
        }
        NodeKind::Future {
            interval: _,
            operand,
        } if is_false(state, operand) => {
            applied_kind(state, "temporal.future.false", NodeKind::False, span)
        }
        NodeKind::Future {
            interval: _,
            operand,
        } if is_true(state, operand) => {
            applied_kind(state, "temporal.future.true", NodeKind::True, span)
        }
        NodeKind::Globally { interval, operand }
            if interval.start() == 0 && interval.end() == 0 =>
        {
            applied("temporal.globally.singleton", operand)
        }
        NodeKind::Globally {
            interval: _,
            operand,
        } if is_false(state, operand) => {
            applied_kind(state, "temporal.globally.false", NodeKind::False, span)
        }
        NodeKind::Globally {
            interval: _,
            operand,
        } if is_true(state, operand) => {
            applied_kind(state, "temporal.globally.true", NodeKind::True, span)
        }
        NodeKind::Until {
            interval,
            left: _,
            right,
        } if interval.start() == 0 && interval.end() == 0 => {
            applied("temporal.until.singleton", right)
        }
        NodeKind::Until {
            interval,
            left,
            right,
        } if is_true(state, left) => applied_kind(
            state,
            "temporal.until.true-left",
            NodeKind::Future {
                interval,
                operand: right,
            },
            span,
        ),
        NodeKind::Release {
            interval,
            left: _,
            right,
        } if interval.start() == 0 && interval.end() == 0 => {
            applied("temporal.release.singleton", right)
        }
        NodeKind::Release {
            interval,
            left,
            right,
        } if is_false(state, left) => applied_kind(
            state,
            "temporal.release.false-left",
            NodeKind::Globally {
                interval,
                operand: right,
            },
            span,
        ),
        _ => Ok(None),
    }
}

fn build_pass(
    input: &FormulaDocument,
    pass: u32,
    state: &mut PassState,
    steps: &mut Vec<RewriteStep>,
    rolling: &mut String,
) -> Result<FormulaDocument, Abort> {
    struct PendingStep {
        source_node: u32,
        source_span: Option<SourceSpan>,
        rule_id: &'static str,
        rule_revision: u32,
        before_sha256: String,
        after_sha256: String,
    }

    state.nodes.clear();
    state.interner.clear();
    let mut mapped = Vec::with_capacity(input.nodes().len());
    let mut retained_source_operands = Vec::with_capacity(input.nodes().len());
    let mut pending_steps = Vec::new();
    for (index, node) in input.nodes().iter().enumerate() {
        state.work()?;
        let before = remap(node.kind, &mapped);
        let replacement = apply_first(state, before, node.span)?;
        let output = if let Some((rule_id, rule_revision, output)) = replacement {
            let before_sha256 = sha256_json(&before);
            // Step identities feed the rolling replay fingerprint.  Preserve the
            // node span as diagnostic provenance, but never let it change that
            // semantic identity.
            let after_sha256 = sha256_json(&state.nodes[output.0 as usize].kind);
            pending_steps.push(PendingStep {
                source_node: index as u32,
                source_span: node.span,
                rule_id,
                rule_revision,
                before_sha256,
                after_sha256,
            });
            retained_source_operands.push(retained_operands(rule_id, node.kind));
            output
        } else {
            retained_source_operands.push(operands(node.kind));
            state.emit(before, node.span)?
        };
        mapped.push(output);
    }
    let root = mapped[input.root().0 as usize];
    let mut relevant_sources = vec![false; input.nodes().len()];
    let mut source_stack = vec![input.root()];
    while let Some(source) = source_stack.pop() {
        let index = source.0 as usize;
        if relevant_sources[index] {
            continue;
        }
        state.work()?;
        relevant_sources[index] = true;
        source_stack.extend(retained_source_operands[index].into_iter().flatten());
    }
    for pending in pending_steps
        .into_iter()
        .filter(|pending| relevant_sources[pending.source_node as usize])
    {
        if steps.len() as u64 >= state.budgets.max_rule_applications {
            return Err(Abort::Budget(BudgetKind::RuleApplications));
        }
        *rolling = sha256_bytes(
            format!(
                "{}\0{}\0{}\0{}\0{}",
                rolling,
                pending.rule_id,
                pending.rule_revision,
                pending.before_sha256,
                pending.after_sha256
            )
            .as_bytes(),
        );
        steps.push(RewriteStep {
            sequence: steps.len() as u64,
            pass,
            source_node: pending.source_node,
            source_span: pending.source_span,
            rule_id: pending.rule_id.to_owned(),
            rule_revision: pending.rule_revision,
            before_sha256: pending.before_sha256,
            after_sha256: pending.after_sha256,
            intermediate_sha256: rolling.clone(),
        });
    }
    let document = FormulaDocument::new(input.semantic_profile(), root, state.nodes.clone())
        .map_err(|_| Abort::Budget(BudgetKind::Nodes))?;
    let compacted = compact_document(&document).ok_or(Abort::Budget(BudgetKind::Nodes))?;
    if compacted.nodes().len() > state.budgets.max_nodes as usize {
        return Err(Abort::Budget(BudgetKind::Nodes));
    }
    Ok(compacted)
}

fn observe_state(seen: &mut BTreeSet<String>, digest: String) -> Result<(), RewriteStatus> {
    if seen.insert(digest) {
        Ok(())
    } else {
        Err(RewriteStatus::NonConvergent)
    }
}

fn report_base(
    input: &FormulaDocument,
    formula_id: String,
    options: RewriteOptions,
    source_revision: String,
) -> RewriteReport {
    let catalog_sha256 = catalog().catalog_sha256;
    let request_sha256 = sha256_json(&(
        input.semantic_view(),
        &formula_id,
        options,
        &source_revision,
        &catalog_sha256,
    ));
    RewriteReport {
        schema_version: "tl-rewrite.report/v1".to_owned(),
        formula_id,
        engine_source_revision: source_revision,
        syntax_revision: TL_SYNTAX_REVISION.to_owned(),
        catalog_sha256,
        input_sha256: sha256_json(&input.semantic_view()),
        request_sha256,
        signal_catalog_sha256: None,
        requirement_context: None,
        binding_failure: None,
        output_sha256: None,
        partial_sha256: None,
        semantic_profile: input.semantic_profile().as_str().to_owned(),
        options,
        status: RewriteStatus::InvalidInput,
        exhausted_budget: None,
        detail: None,
        iterations: 0,
        work_units: 0,
        rule_applications: 0,
        steps: Vec::new(),
        output: None,
    }
}

fn contextual_report_base(
    input: &FormulaDocument,
    formula_id: String,
    options: RewriteOptions,
    source_revision: String,
    signal_catalog: &SignalCatalogDocument,
    requirement_context: Option<RequirementContextDocument>,
) -> RewriteReport {
    let catalog_sha256 = catalog().catalog_sha256;
    let signal_catalog_sha256 = sha256_json(signal_catalog);
    let request_sha256 = sha256_json(&(
        "tl-rewrite.contextual-request/v2",
        input,
        &formula_id,
        options,
        &source_revision,
        &catalog_sha256,
        signal_catalog,
        &requirement_context,
        TL_SYNTAX_REVISION,
    ));
    RewriteReport {
        schema_version: "tl-rewrite.report/v2".to_owned(),
        formula_id,
        engine_source_revision: source_revision,
        syntax_revision: TL_SYNTAX_REVISION.to_owned(),
        catalog_sha256,
        input_sha256: sha256_json(input),
        request_sha256,
        signal_catalog_sha256: Some(signal_catalog_sha256),
        requirement_context: Some(requirement_context),
        binding_failure: None,
        output_sha256: None,
        partial_sha256: None,
        semantic_profile: input.semantic_profile().as_str().to_owned(),
        options,
        status: RewriteStatus::InvalidInput,
        exhausted_budget: None,
        detail: None,
        iterations: 0,
        work_units: 0,
        rule_applications: 0,
        steps: Vec::new(),
        output: None,
    }
}

pub(crate) fn binding_check(
    signal_catalog: SignalCatalog<'_>,
    formula: Formula<'_>,
    locus: BindingLocus,
) -> BindingCheck {
    match signal_catalog.bind_formula(formula) {
        Err(FormulaBindingError::MissingPropositionBinding { proposition }) => {
            BindingCheck::Missing(BindingFailure {
                locus,
                proposition_id: proposition.0,
            })
        }
        Ok(_) => BindingCheck::Bound,
        Err(error) => BindingCheck::Refused(error.to_string()),
    }
}

fn run_passes<F>(
    mut current: FormulaDocument,
    input_was_compacted: bool,
    mut report: RewriteReport,
    options: RewriteOptions,
    mut state: PassState,
    mut build: F,
) -> RewriteReport
where
    F: FnMut(
        &FormulaDocument,
        u32,
        &mut PassState,
        &mut Vec<RewriteStep>,
        &mut String,
    ) -> Result<FormulaDocument, Abort>,
{
    let mut seen = BTreeSet::from([sha256_json(&current.semantic_view())]);
    let mut steps = Vec::new();
    let mut rolling = sha256_bytes(b"tl-rewrite.trace/v1");

    for pass in 0..options.budgets.max_iterations {
        let candidate = match build(&current, pass, &mut state, &mut steps, &mut rolling) {
            Ok(candidate) => candidate,
            Err(Abort::Budget(budget)) => {
                report.status = RewriteStatus::BudgetExhausted;
                report.exhausted_budget = Some(budget);
                report.partial_sha256 = Some(sha256_json(&current.semantic_view()));
                report.detail = Some(format!("rewrite exhausted {budget:?} budget"));
                report.work_units = state.work_units;
                report.rule_applications = steps.len() as u64;
                report.steps = steps;
                return report;
            }
        };
        report.iterations = pass + 1;
        let candidate_sha256 = sha256_json(&candidate.semantic_view());
        if candidate == current {
            report.status = if steps.is_empty() && !input_was_compacted {
                RewriteStatus::Unchanged
            } else {
                RewriteStatus::Normalized
            };
            report.output_sha256 = Some(candidate_sha256);
            report.work_units = state.work_units;
            report.rule_applications = steps.len() as u64;
            report.steps = steps;
            report.output = Some(candidate);
            return report;
        }
        if observe_state(&mut seen, candidate_sha256.clone()).is_err() {
            report.status = RewriteStatus::NonConvergent;
            report.partial_sha256 = Some(candidate_sha256);
            report.detail = Some("a prior complete formula state reappeared".to_owned());
            report.work_units = state.work_units;
            report.rule_applications = steps.len() as u64;
            report.steps = steps;
            return report;
        }
        current = candidate;
    }

    report.status = RewriteStatus::BudgetExhausted;
    report.exhausted_budget = Some(BudgetKind::Iterations);
    report.partial_sha256 = Some(sha256_json(&current.semantic_view()));
    report.detail = Some("rewrite exhausted Iterations budget".to_owned());
    report.work_units = state.work_units;
    report.rule_applications = steps.len() as u64;
    report.steps = steps;
    report
}

/// Rewrites one validated formula to a fixed point or explicit non-success.
///
/// Implements: FR-002, FR-003
fn rewrite_validated_with_builder<F>(
    input: &FormulaDocument,
    mut report: RewriteReport,
    options: RewriteOptions,
    build: F,
) -> RewriteReport
where
    F: FnMut(
        &FormulaDocument,
        u32,
        &mut PassState,
        &mut Vec<RewriteStep>,
        &mut String,
    ) -> Result<FormulaDocument, Abort>,
{
    if input.semantic_profile() != SemanticProfile::ClosedTraceV1 {
        report.status = RewriteStatus::UnsupportedProfile;
        report.detail = Some(
            "the v1 rule catalog is enabled only for mltl.closed-trace/v1; online-prefix evidence remains pending"
                .to_owned(),
        );
        return report;
    }

    let Some(current) = compact_document(input) else {
        report.detail = Some("validated input could not be compacted".to_owned());
        return report;
    };
    let input_was_compacted = current != *input;
    if current.nodes().len() > options.budgets.max_nodes as usize {
        report.status = RewriteStatus::BudgetExhausted;
        report.exhausted_budget = Some(BudgetKind::Nodes);
        report.partial_sha256 = Some(sha256_json(&current.semantic_view()));
        report.detail = Some("rewrite exhausted Nodes budget".to_owned());
        return report;
    }
    let state = PassState {
        nodes: Vec::new(),
        interner: BTreeMap::new(),
        work_units: 0,
        budgets: options.budgets,
    };
    run_passes(current, input_was_compacted, report, options, state, build)
}

/// Rewrites one validated formula to a fixed point or explicit non-success.
///
/// Implements: FR-002, FR-003
pub fn rewrite(
    input: &FormulaDocument,
    formula_id: impl Into<String>,
    options: RewriteOptions,
    source_revision: impl Into<String>,
) -> RewriteReport {
    let mut report = report_base(input, formula_id.into(), options, source_revision.into());
    if let Err(error) = input.validate() {
        report.detail = Some(error.to_string());
        return report;
    }
    rewrite_validated_with_builder(input, report, options, build_pass)
}

/// Rewrites with one shared signal catalog and an exact-or-absent caller context.
///
/// The catalog is validated before any rewrite work. A missing formula binding is
/// a typed v2 non-success; catalog shape validation itself remains owned by
/// `tl-syntax`.
///
/// Implements: FR-007-AC-1, FR-007-AC-3
pub fn rewrite_with_context(
    input: &FormulaDocument,
    formula_id: impl Into<String>,
    options: RewriteOptions,
    source_revision: impl Into<String>,
    signal_catalog: &SignalCatalogDocument,
    requirement_context: Option<RequirementContextDocument>,
) -> RewriteReport {
    rewrite_with_context_using_builder(
        input,
        formula_id,
        options,
        source_revision,
        signal_catalog,
        requirement_context,
        build_pass,
    )
}

fn rewrite_with_context_using_builder<F>(
    input: &FormulaDocument,
    formula_id: impl Into<String>,
    options: RewriteOptions,
    source_revision: impl Into<String>,
    signal_catalog: &SignalCatalogDocument,
    requirement_context: Option<RequirementContextDocument>,
    build: F,
) -> RewriteReport
where
    F: FnMut(
        &FormulaDocument,
        u32,
        &mut PassState,
        &mut Vec<RewriteStep>,
        &mut String,
    ) -> Result<FormulaDocument, Abort>,
{
    let formula_id = formula_id.into();
    let source_revision = source_revision.into();
    let mut contextual = contextual_report_base(
        input,
        formula_id.clone(),
        options,
        source_revision.clone(),
        signal_catalog,
        requirement_context,
    );
    let Ok(formula) = input.validate() else {
        contextual.detail = Some("the input formula is invalid".to_owned());
        return contextual;
    };
    let Ok(catalog) = signal_catalog.validate() else {
        contextual.detail = Some("the supplied signal catalog is invalid".to_owned());
        return contextual;
    };
    match binding_check(catalog, formula, BindingLocus::Input) {
        BindingCheck::Bound => {}
        BindingCheck::Missing(binding_failure) => {
            contextual.status = RewriteStatus::UnresolvedBinding;
            contextual.binding_failure = Some(binding_failure);
            contextual.detail = Some(format!(
                "formula proposition {} has no signal binding",
                binding_failure.proposition_id
            ));
            return contextual;
        }
        BindingCheck::Refused(error) => {
            contextual.status = RewriteStatus::UnresolvedBinding;
            contextual.detail = Some(format!(
                "the supplied signal catalog refused input binding: {error}"
            ));
            return contextual;
        }
    }

    let mut observed = rewrite_validated_with_builder(input, contextual, options, build);
    if let Some(output) = observed.output.as_ref() {
        match output.validate() {
            Ok(output_formula) => {
                match binding_check(catalog, output_formula, BindingLocus::Output) {
                    BindingCheck::Bound => {}
                    BindingCheck::Missing(binding_failure) => {
                        observed.status = RewriteStatus::UnresolvedBinding;
                        observed.binding_failure = Some(binding_failure);
                        observed.detail = Some(format!(
                            "formula proposition {} has no signal binding",
                            binding_failure.proposition_id
                        ));
                        observed.output = None;
                        observed.output_sha256 = None;
                    }
                    BindingCheck::Refused(error) => {
                        observed.status = RewriteStatus::UnresolvedBinding;
                        observed.detail = Some(format!(
                            "the supplied signal catalog refused output binding: {error}"
                        ));
                        observed.output = None;
                        observed.output_sha256 = None;
                    }
                }
            }
            Err(error) => {
                observed.status = RewriteStatus::InvalidInput;
                observed.detail = Some(format!("a successful rewrite output was invalid: {error}"));
                observed.output = None;
                observed.output_sha256 = None;
            }
        }
    }
    observed
}

/// Re-executes a report's exact identity and options and compares every field.
///
/// Implements: FR-003
pub fn replay(input: &FormulaDocument, expected: &RewriteReport) -> ReplayReport {
    let observed = rewrite(
        input,
        expected.formula_id.clone(),
        expected.options,
        expected.engine_source_revision.clone(),
    );
    let expected_report_sha256 = sha256_json(expected);
    let observed_report_sha256 = sha256_json(&observed);
    ReplayReport {
        schema_version: "tl-rewrite.replay/v1".to_owned(),
        status: if observed == *expected {
            ReplayStatus::Verified
        } else {
            ReplayStatus::Mismatch
        },
        expected_report_sha256,
        observed_report_sha256,
        signal_catalog_sha256: None,
        requirement_context: None,
    }
}

/// Re-executes a contextual request and compares the complete native v2 report.
///
/// Callers must resupply the complete catalog and exact-or-absent context; their
/// values are inputs to the observed request digest rather than copied from the
/// expected record.
///
/// Implements: FR-007-AC-2
pub fn replay_with_context(
    input: &FormulaDocument,
    expected: &RewriteReport,
    signal_catalog: &SignalCatalogDocument,
    requirement_context: Option<RequirementContextDocument>,
) -> ReplayReport {
    let observed = rewrite_with_context(
        input,
        expected.formula_id.clone(),
        expected.options,
        expected.engine_source_revision.clone(),
        signal_catalog,
        requirement_context.clone(),
    );
    let expected_report_sha256 = sha256_json(expected);
    let observed_report_sha256 = sha256_json(&observed);
    ReplayReport {
        schema_version: "tl-rewrite.replay/v2".to_owned(),
        status: if observed == *expected {
            ReplayStatus::Verified
        } else {
            ReplayStatus::Mismatch
        },
        expected_report_sha256,
        observed_report_sha256,
        signal_catalog_sha256: Some(sha256_json(signal_catalog)),
        requirement_context: Some(requirement_context),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        report_base, rewrite_with_context_using_builder, run_passes, BindingFailure, BindingLocus,
        PassState, RewriteOptions, RewriteStatus,
    };
    use std::collections::BTreeMap;
    use tl_syntax::{
        FormulaDocument, Node, NodeId, NodeKind, OwnedSignalDeclaration, PropositionBinding,
        PropositionId, SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId, SourceSpan,
    };

    // Trace: TC-020, FR-002-AC-2, NFR-001-AC-2
    #[test]
    fn repeated_complete_state_through_engine_is_non_convergent() {
        let document = |kind, start| {
            FormulaDocument::new(
                SemanticProfile::ClosedTraceV1,
                NodeId(0),
                vec![Node::with_span(
                    kind,
                    SourceSpan::new(start, start + 1).unwrap(),
                )],
            )
            .unwrap()
        };
        let first = document(NodeKind::True, 0);
        let second = document(NodeKind::False, 2);
        let repeated_first = document(NodeKind::True, 4);
        let options = RewriteOptions::default();
        let report = report_base(&first, "cycle".to_owned(), options, "source".to_owned());
        let state = PassState {
            nodes: Vec::new(),
            interner: BTreeMap::new(),
            work_units: 0,
            budgets: options.budgets,
        };
        let observed = run_passes(
            first.clone(),
            false,
            report,
            options,
            state,
            |_, pass, _, _, _| {
                Ok(if pass % 2 == 0 {
                    second.clone()
                } else {
                    repeated_first.clone()
                })
            },
        );
        assert_eq!(observed.status, RewriteStatus::NonConvergent);
        assert_eq!(observed.iterations, 2);
        assert!(observed.output.is_none());
    }

    // Trace: TC-033, FR-007-AC-3
    #[test]
    fn output_binding_refusal_names_the_output_locus() {
        let input = FormulaDocument::new(
            SemanticProfile::ClosedTraceV1,
            NodeId(0),
            vec![Node::new(NodeKind::Proposition {
                proposition: PropositionId(7),
            })],
        )
        .unwrap();
        let output = FormulaDocument::new(
            SemanticProfile::ClosedTraceV1,
            NodeId(0),
            vec![Node::new(NodeKind::Proposition {
                proposition: PropositionId(8),
            })],
        )
        .unwrap();
        let signal_catalog = SignalCatalogDocument::new(
            vec![OwnedSignalDeclaration::new(
                SignalId(11),
                "request_ready".to_owned(),
                SignalDomain::Boolean,
            )],
            vec![PropositionBinding::new(PropositionId(7), SignalId(11))],
        )
        .unwrap();
        let observed = rewrite_with_context_using_builder(
            &input,
            "output-binding",
            RewriteOptions::default(),
            "source",
            &signal_catalog,
            None,
            |_, _, _, _, _| Ok(output.clone()),
        );
        assert_eq!(observed.status, RewriteStatus::UnresolvedBinding);
        assert_eq!(
            observed.binding_failure,
            Some(BindingFailure {
                locus: BindingLocus::Output,
                proposition_id: 8,
            })
        );
        assert!(observed.output.is_none());
        assert!(observed.output_sha256.is_none());
    }
}
