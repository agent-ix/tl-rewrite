use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use tl_syntax::{FormulaDocument, Node, NodeId, NodeKind, SemanticProfile, SourceSpan};

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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Abort {
    Budget(BudgetKind),
}

struct PassState {
    nodes: Vec<Node>,
    work_units: u64,
    applications: u64,
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

    fn application(&mut self) -> Result<(), Abort> {
        if self.applications >= self.budgets.max_rule_applications {
            return Err(Abort::Budget(BudgetKind::RuleApplications));
        }
        self.applications = self
            .applications
            .checked_add(1)
            .ok_or(Abort::Budget(BudgetKind::RuleApplications))?;
        Ok(())
    }

    fn emit(&mut self, kind: NodeKind, span: Option<SourceSpan>) -> Result<NodeId, Abort> {
        self.work()?;
        if self.nodes.len() >= self.budgets.max_nodes as usize {
            return Err(Abort::Budget(BudgetKind::Nodes));
        }
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(Node { kind, span });
        Ok(id)
    }

    fn kind(&self, id: NodeId) -> NodeKind {
        self.nodes[id.0 as usize].kind
    }
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
    state: &mut PassState,
    rule: &'static str,
    replacement: NodeId,
) -> Result<Option<(&'static str, NodeId)>, Abort> {
    state.application()?;
    Ok(Some((rule, replacement)))
}

fn applied_kind(
    state: &mut PassState,
    rule: &'static str,
    kind: NodeKind,
    span: Option<SourceSpan>,
) -> Result<Option<(&'static str, NodeId)>, Abort> {
    state.application()?;
    let replacement = state.emit(kind, span)?;
    Ok(Some((rule, replacement)))
}

fn apply_not(
    state: &mut PassState,
    operand: NodeId,
    span: Option<SourceSpan>,
) -> Result<Option<(&'static str, NodeId)>, Abort> {
    match state.kind(operand) {
        NodeKind::False => applied_kind(state, "bool.not.false", NodeKind::True, span),
        NodeKind::True => applied_kind(state, "bool.not.true", NodeKind::False, span),
        NodeKind::Not { operand } => applied(state, "bool.not.double", operand),
        NodeKind::Future { interval, operand } => {
            state.application()?;
            let negated = state.emit(NodeKind::Not { operand }, None)?;
            let output = state.emit(
                NodeKind::Globally {
                    interval,
                    operand: negated,
                },
                span,
            )?;
            Ok(Some(("neg.future.dual", output)))
        }
        NodeKind::Globally { interval, operand } => {
            state.application()?;
            let negated = state.emit(NodeKind::Not { operand }, None)?;
            let output = state.emit(
                NodeKind::Future {
                    interval,
                    operand: negated,
                },
                span,
            )?;
            Ok(Some(("neg.globally.dual", output)))
        }
        NodeKind::Until {
            interval,
            left,
            right,
        } => {
            state.application()?;
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
            Ok(Some(("neg.until.dual", output)))
        }
        NodeKind::Release {
            interval,
            left,
            right,
        } => {
            state.application()?;
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
            Ok(Some(("neg.release.dual", output)))
        }
        _ => Ok(None),
    }
}

fn apply_first(
    state: &mut PassState,
    kind: NodeKind,
    span: Option<SourceSpan>,
) -> Result<Option<(&'static str, NodeId)>, Abort> {
    match kind {
        NodeKind::Not { operand } => apply_not(state, operand, span),
        NodeKind::And { left, right: _ } if is_false(state, left) => {
            applied_kind(state, "bool.and.false-left", NodeKind::False, span)
        }
        NodeKind::And { left: _, right } if is_false(state, right) => {
            applied_kind(state, "bool.and.false-right", NodeKind::False, span)
        }
        NodeKind::And { left, right } if is_true(state, left) => {
            applied(state, "bool.and.true-left", right)
        }
        NodeKind::And { left, right } if is_true(state, right) => {
            applied(state, "bool.and.true-right", left)
        }
        NodeKind::And { left, right } if left == right => {
            applied(state, "bool.and.idempotent", left)
        }
        NodeKind::Or { left, right: _ } if is_true(state, left) => {
            applied_kind(state, "bool.or.true-left", NodeKind::True, span)
        }
        NodeKind::Or { left: _, right } if is_true(state, right) => {
            applied_kind(state, "bool.or.true-right", NodeKind::True, span)
        }
        NodeKind::Or { left, right } if is_false(state, left) => {
            applied(state, "bool.or.false-left", right)
        }
        NodeKind::Or { left, right } if is_false(state, right) => {
            applied(state, "bool.or.false-right", left)
        }
        NodeKind::Or { left, right } if left == right => applied(state, "bool.or.idempotent", left),
        NodeKind::Implies { left, right: _ } if is_false(state, left) => {
            applied_kind(state, "bool.implies.false-left", NodeKind::True, span)
        }
        NodeKind::Implies { left, right } if is_true(state, left) => {
            applied(state, "bool.implies.true-left", right)
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
            state.application()?;
            let negated = state.emit(NodeKind::Not { operand: left }, None)?;
            let output = state.emit(
                NodeKind::Or {
                    left: negated,
                    right,
                },
                span,
            )?;
            Ok(Some(("bool.implies.eliminate", output)))
        }
        NodeKind::Equivalent { left, right } if left == right => {
            applied_kind(state, "bool.equivalent.reflexive", NodeKind::True, span)
        }
        NodeKind::Equivalent { left, right } if is_true(state, left) => {
            applied(state, "bool.equivalent.true-left", right)
        }
        NodeKind::Equivalent { left, right } if is_true(state, right) => {
            applied(state, "bool.equivalent.true-right", left)
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
            applied(state, "temporal.future.singleton", operand)
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
            applied(state, "temporal.globally.singleton", operand)
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
            applied(state, "temporal.until.singleton", right)
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
            applied(state, "temporal.release.singleton", right)
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
    state.nodes.clear();
    let mut mapped = Vec::with_capacity(input.nodes.len());
    for (index, node) in input.nodes.iter().enumerate() {
        state.work()?;
        let before = remap(node.kind, &mapped);
        let replacement = apply_first(state, before, node.span)?;
        let output = if let Some((rule_id, output)) = replacement {
            let before_sha256 = sha256_json(&before);
            let after_sha256 = sha256_json(&state.nodes[output.0 as usize]);
            *rolling = sha256_bytes(
                format!(
                    "{}\0{}\01\0{}\0{}",
                    rolling, rule_id, before_sha256, after_sha256
                )
                .as_bytes(),
            );
            steps.push(RewriteStep {
                sequence: steps.len() as u64,
                pass,
                source_node: index as u32,
                source_span: node.span,
                rule_id: rule_id.to_owned(),
                rule_revision: 1,
                before_sha256,
                after_sha256,
                intermediate_sha256: rolling.clone(),
            });
            output
        } else {
            state.emit(before, node.span)?
        };
        mapped.push(output);
    }
    FormulaDocument::new(
        input.semantic_profile,
        mapped[input.root.0 as usize],
        state.nodes.clone(),
    )
    .map_err(|_| Abort::Budget(BudgetKind::Nodes))
}

fn report_base(
    input: &FormulaDocument,
    formula_id: String,
    options: RewriteOptions,
    source_revision: String,
) -> RewriteReport {
    let catalog_sha256 = catalog().catalog_sha256;
    let request_sha256 = sha256_json(&(
        input,
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
        input_sha256: sha256_json(input),
        request_sha256,
        output_sha256: None,
        partial_sha256: None,
        semantic_profile: input.semantic_profile.as_str().to_owned(),
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
    if input.semantic_profile != SemanticProfile::ClosedTraceV1 {
        report.status = RewriteStatus::UnsupportedProfile;
        report.detail = Some(
            "the v1 rule catalog is enabled only for mltl.closed-trace/v1; online-prefix evidence remains pending"
                .to_owned(),
        );
        return report;
    }

    let mut current = input.clone();
    let mut seen = BTreeSet::from([sha256_json(&current)]);
    let mut state = PassState {
        nodes: Vec::new(),
        work_units: 0,
        applications: 0,
        budgets: options.budgets,
    };
    let mut steps = Vec::new();
    let mut rolling = sha256_bytes(b"tl-rewrite.trace/v1");

    for pass in 0..options.budgets.max_iterations {
        let candidate = match build_pass(&current, pass, &mut state, &mut steps, &mut rolling) {
            Ok(candidate) => candidate,
            Err(Abort::Budget(budget)) => {
                report.status = RewriteStatus::BudgetExhausted;
                report.exhausted_budget = Some(budget);
                report.partial_sha256 = Some(sha256_json(&current));
                report.detail = Some(format!("rewrite exhausted {budget:?} budget"));
                report.work_units = state.work_units;
                report.rule_applications = state.applications;
                report.steps = steps;
                return report;
            }
        };
        report.iterations = pass + 1;
        let candidate_sha256 = sha256_json(&candidate);
        if candidate == current {
            report.status = if steps.is_empty() {
                RewriteStatus::Unchanged
            } else {
                RewriteStatus::Normalized
            };
            report.output_sha256 = Some(candidate_sha256);
            report.work_units = state.work_units;
            report.rule_applications = state.applications;
            report.steps = steps;
            report.output = Some(candidate);
            return report;
        }
        if !seen.insert(candidate_sha256.clone()) {
            report.status = RewriteStatus::NonConvergent;
            report.partial_sha256 = Some(candidate_sha256);
            report.detail = Some("a prior complete formula state reappeared".to_owned());
            report.work_units = state.work_units;
            report.rule_applications = state.applications;
            report.steps = steps;
            return report;
        }
        current = candidate;
    }

    report.status = RewriteStatus::BudgetExhausted;
    report.exhausted_budget = Some(BudgetKind::Iterations);
    report.partial_sha256 = Some(sha256_json(&current));
    report.detail = Some("rewrite exhausted Iterations budget".to_owned());
    report.work_units = state.work_units;
    report.rule_applications = state.applications;
    report.steps = steps;
    report
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
    }
}
