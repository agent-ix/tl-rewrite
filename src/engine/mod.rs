//! Shared deterministic rewrite traversal and resource accounting.

mod boolean;
pub mod future;
pub mod past;

use std::collections::{BTreeMap, BTreeSet};

use tl_syntax::{
    Formula, FormulaBindingError, FormulaDocument, FormulaSchemaVersion, Node, NodeId, NodeKind,
    RequirementContextDocument, SignalCatalog, SignalCatalogDocument, SourceSpan,
    SyntaxArtifactLimits,
};

use crate::{
    catalog::catalog_for_profile,
    hash::sha256_bytes,
    hash::sha256_json,
    report::{
        BindingFailure, BindingLocus, BudgetKind, RewriteBudgets, RewriteOptions, RewriteReport,
        RewriteStatus, RewriteStep,
    },
    TL_SYNTAX_REVISION,
};

/// Closed local interpretation of one shared-catalog binding attempt.
///
/// `FormulaBindingError` is non-exhaustive upstream. New refusal variants must
/// remain non-success here instead of silently becoming an accepted binding.
pub(crate) enum BindingCheck {
    Bound,
    Missing(BindingFailure),
    Refused(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Abort {
    Budget(BudgetKind),
    OwnerAdmission(String),
    UnsupportedProfile,
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
        let id =
            NodeId(u32::try_from(self.nodes.len()).map_err(|_| Abort::Budget(BudgetKind::Nodes))?);
        self.nodes.push(Node { kind, span });
        self.interner.insert(kind, id);
        Ok(id)
    }

    fn kind(&self, id: NodeId) -> Option<NodeKind> {
        usize::try_from(id.0)
            .ok()
            .and_then(|index| self.nodes.get(index))
            .map(|node| node.kind)
    }
}

fn operands(kind: NodeKind) -> [Option<NodeId>; 2] {
    match kind {
        NodeKind::Not { operand }
        | NodeKind::Future { operand, .. }
        | NodeKind::Globally { operand, .. }
        | NodeKind::Once { operand, .. }
        | NodeKind::Historically { operand, .. }
        | NodeKind::StrongPrevious { operand } => [Some(operand), None],
        NodeKind::And { left, right }
        | NodeKind::Or { left, right }
        | NodeKind::Implies { left, right }
        | NodeKind::Equivalent { left, right }
        | NodeKind::Until { left, right, .. }
        | NodeKind::Release { left, right, .. }
        | NodeKind::Since { left, right, .. }
        | NodeKind::Triggered { left, right, .. } => [Some(left), Some(right)],
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
        let Ok(index) = usize::try_from(id.0) else {
            continue;
        };
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
        let kind = remap(node.kind, &mapped)?;
        let id = if let Some(existing) = interner.get(&kind) {
            *existing
        } else {
            let id = NodeId(u32::try_from(nodes.len()).ok()?);
            nodes.push(Node {
                kind,
                span: node.span,
            });
            interner.insert(kind, id);
            id
        };
        mapped[index] = id;
    }
    let root = *mapped.get(usize::try_from(input.root().0).ok()?)?;
    rebuild_document(input, root, nodes).ok()
}

fn count_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn max_nodes_usize(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

fn rebuild_document(
    input: &FormulaDocument,
    root: NodeId,
    nodes: Vec<Node>,
) -> Result<FormulaDocument, tl_syntax::FormulaError> {
    match input.schema_version() {
        FormulaSchemaVersion::V1 => FormulaDocument::new(input.semantic_profile(), root, nodes),
        FormulaSchemaVersion::V2 => FormulaDocument::new_v2(input.semantic_profile(), root, nodes),
    }
}

fn remap(kind: NodeKind, mapped: &[NodeId]) -> Option<NodeKind> {
    let id = |value: NodeId| {
        usize::try_from(value.0)
            .ok()
            .and_then(|index| mapped.get(index))
            .copied()
    };
    Some(match kind {
        NodeKind::False => NodeKind::False,
        NodeKind::True => NodeKind::True,
        NodeKind::Proposition { proposition } => NodeKind::Proposition { proposition },
        NodeKind::Not { operand } => NodeKind::Not {
            operand: id(operand)?,
        },
        NodeKind::And { left, right } => NodeKind::And {
            left: id(left)?,
            right: id(right)?,
        },
        NodeKind::Or { left, right } => NodeKind::Or {
            left: id(left)?,
            right: id(right)?,
        },
        NodeKind::Implies { left, right } => NodeKind::Implies {
            left: id(left)?,
            right: id(right)?,
        },
        NodeKind::Equivalent { left, right } => NodeKind::Equivalent {
            left: id(left)?,
            right: id(right)?,
        },
        NodeKind::Future { interval, operand } => NodeKind::Future {
            interval,
            operand: id(operand)?,
        },
        NodeKind::Globally { interval, operand } => NodeKind::Globally {
            interval,
            operand: id(operand)?,
        },
        NodeKind::Until {
            interval,
            left,
            right,
        } => NodeKind::Until {
            interval,
            left: id(left)?,
            right: id(right)?,
        },
        NodeKind::Release {
            interval,
            left,
            right,
        } => NodeKind::Release {
            interval,
            left: id(left)?,
            right: id(right)?,
        },
        NodeKind::Once { interval, operand } => NodeKind::Once {
            interval,
            operand: id(operand)?,
        },
        NodeKind::Historically { interval, operand } => NodeKind::Historically {
            interval,
            operand: id(operand)?,
        },
        NodeKind::StrongPrevious { operand } => NodeKind::StrongPrevious {
            operand: id(operand)?,
        },
        NodeKind::Since {
            interval,
            left,
            right,
        } => NodeKind::Since {
            interval,
            left: id(left)?,
            right: id(right)?,
        },
        NodeKind::Triggered {
            interval,
            left,
            right,
        } => NodeKind::Triggered {
            interval,
            left: id(left)?,
            right: id(right)?,
        },
    })
}

fn is_false(state: &PassState, id: NodeId) -> bool {
    state.kind(id) == Some(NodeKind::False)
}

fn is_true(state: &PassState, id: NodeId) -> bool {
    state.kind(id) == Some(NodeKind::True)
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
        let before = remap(node.kind, &mapped).ok_or(Abort::Budget(BudgetKind::Nodes))?;
        let replacement = if future::supports(input) {
            future::apply_first(state, before, node.span)?
        } else if past::supports(input) {
            past::apply_first(state, before, node.span)?
        } else {
            return Err(Abort::UnsupportedProfile);
        };
        let output = if let Some((rule_id, rule_revision, output)) = replacement {
            let before_sha256 = sha256_json(&before);
            // Step identities feed the rolling replay fingerprint.  Preserve the
            // node span as diagnostic provenance, but never let it change that
            // semantic identity.
            let output_index =
                usize::try_from(output.0).map_err(|_| Abort::Budget(BudgetKind::Nodes))?;
            let output_kind = state
                .nodes
                .get(output_index)
                .ok_or(Abort::Budget(BudgetKind::Nodes))?
                .kind;
            let after_sha256 = sha256_json(&output_kind);
            let source_node = u32::try_from(index).map_err(|_| Abort::Budget(BudgetKind::Nodes))?;
            pending_steps.push(PendingStep {
                source_node,
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
    let root_index =
        usize::try_from(input.root().0).map_err(|_| Abort::Budget(BudgetKind::Nodes))?;
    let root = *mapped
        .get(root_index)
        .ok_or(Abort::Budget(BudgetKind::Nodes))?;
    let mut relevant_sources = vec![false; input.nodes().len()];
    let mut source_stack = vec![input.root()];
    while let Some(source) = source_stack.pop() {
        let index = usize::try_from(source.0).map_err(|_| Abort::Budget(BudgetKind::Nodes))?;
        let Some(relevant) = relevant_sources.get_mut(index) else {
            return Err(Abort::Budget(BudgetKind::Nodes));
        };
        if *relevant {
            continue;
        }
        state.work()?;
        *relevant = true;
        let retained = retained_source_operands
            .get(index)
            .ok_or(Abort::Budget(BudgetKind::Nodes))?;
        source_stack.extend(retained.iter().copied().flatten());
    }
    for pending in pending_steps.into_iter().filter(|pending| {
        usize::try_from(pending.source_node)
            .ok()
            .and_then(|index| relevant_sources.get(index))
            .copied()
            .unwrap_or(false)
    }) {
        if count_u64(steps.len()) >= state.budgets.max_rule_applications {
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
            sequence: count_u64(steps.len()),
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
    let document = rebuild_document(input, root, state.nodes.clone())
        .map_err(|_| Abort::Budget(BudgetKind::Nodes))?;
    let compacted = compact_document(&document).ok_or(Abort::Budget(BudgetKind::Nodes))?;
    if compacted.nodes().len() > max_nodes_usize(state.budgets.max_nodes) {
        return Err(Abort::Budget(BudgetKind::Nodes));
    }
    let bytes = compacted
        .canonical_json_bytes()
        .map_err(|error| Abort::OwnerAdmission(error.to_string()))?;
    let limits = SyntaxArtifactLimits {
        formula_nodes: max_nodes_usize(state.budgets.max_nodes),
        ..SyntaxArtifactLimits::default()
    };
    FormulaDocument::from_json_bytes(&bytes, limits)
        .map_err(|error| Abort::OwnerAdmission(error.to_string()))
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
    let catalog_sha256 = catalog_for_profile(input.semantic_profile()).catalog_sha256;
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
    let catalog_sha256 = catalog_for_profile(input.semantic_profile()).catalog_sha256;
    let signal_catalog_sha256 = sha256_json(signal_catalog);
    let request_sha256 = sha256_json(&(
        "tl-rewrite.contextual-request/v2",
        input.semantic_view(),
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
        input_sha256: sha256_json(&input.semantic_view()),
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
                report.rule_applications = count_u64(steps.len());
                report.steps = steps;
                return report;
            }
            Err(Abort::UnsupportedProfile) => {
                report.status = RewriteStatus::UnsupportedProfile;
                report.detail = Some("no profile owner admitted the rewrite pass".to_owned());
                report.work_units = state.work_units;
                report.rule_applications = count_u64(steps.len());
                report.steps = steps;
                return report;
            }
            Err(Abort::OwnerAdmission(detail)) => {
                report.status = RewriteStatus::InvalidInput;
                report.detail = Some(format!(
                    "the syntax owner refused the rebuilt formula: {detail}"
                ));
                report.work_units = state.work_units;
                report.rule_applications = count_u64(steps.len());
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
            report.rule_applications = count_u64(steps.len());
            report.steps = steps;
            report.output = Some(candidate);
            return report;
        }
        if observe_state(&mut seen, candidate_sha256.clone()).is_err() {
            report.status = RewriteStatus::NonConvergent;
            report.partial_sha256 = Some(candidate_sha256);
            report.detail = Some("a prior complete formula state reappeared".to_owned());
            report.work_units = state.work_units;
            report.rule_applications = count_u64(steps.len());
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
    report.rule_applications = count_u64(steps.len());
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
    if !future::supports(input) && !past::supports(input) {
        report.status = RewriteStatus::UnsupportedProfile;
        report.detail = Some(
            "rewrite catalogs are enabled only for mltl.closed-trace/v1 and mltl.origin-complete-history/v1; online-prefix inputs are refused"
                .to_owned(),
        );
        return report;
    }

    let Some(current) = compact_document(input) else {
        report.detail = Some("validated input could not be compacted".to_owned());
        return report;
    };
    let input_was_compacted = current != *input;
    if current.nodes().len() > max_nodes_usize(options.budgets.max_nodes) {
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
