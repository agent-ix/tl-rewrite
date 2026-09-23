//! Independent infinite-profile rule selection and bounded graph rewriting.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use tl_syntax::{
    FairnessPremisesDocument, InfiniteFormulaDocument, InfiniteNode, InfiniteNodeKind, NodeId,
    SourceSpan, TemporalInterval,
};

type Kind = InfiniteNodeKind;

use crate::{
    catalog::infinite_catalog,
    hash::{sha256_bytes, sha256_json},
    report::{
        read_canonical, BudgetKind, RecordLimits, RecordReadError, RewriteBudgets, RewriteOptions,
        RewriteStatus, RewriteStep,
    },
};

/// Owner-lowerable output byte ceiling for one infinite rewrite record.
pub const MAX_INFINITE_REPORT_BYTES: usize = 64 * 1024 * 1024;

/// Typed reason for an infinite rewrite without an output graph.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InfiniteRewriteFailure {
    /// Formula or fairness owner identity did not match the request.
    IdentityMismatch,
    /// Distinct premise roots could not be represented after rewriting.
    UnmappableFairness,
    /// An internal candidate violated the syntax owner's graph invariants.
    Internal,
    /// A configured work or output ceiling was exhausted.
    Budget(BudgetKind),
    /// The canonical output report exceeded its byte ceiling.
    ReportBytes,
    /// A complete graph state repeated before reaching a fixed point.
    NonConvergent,
}

/// Complete, deterministic infinite rewrite attempt. Only success has outputs.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InfiniteRewriteReport {
    /// Exact report edition.
    pub schema_version: String,
    /// Exact input formula document identity.
    pub input_identity: String,
    /// Exact input fairness identity, if supplied.
    pub input_fairness_identity: Option<String>,
    /// Exact source revision supplied by the build.
    pub source_revision: String,
    /// Exact infinite catalog digest.
    pub catalog_sha256: String,
    /// Digest of the complete admitted request.
    pub request_sha256: String,
    /// Replayable traversal and work ceilings.
    pub options: RewriteOptions,
    /// Ceiling for the serialized report, including its output document.
    pub max_report_bytes: usize,
    /// Success or refusal status.
    pub status: RewriteStatus,
    /// Typed failure reason, absent for success.
    pub failure: Option<InfiniteRewriteFailure>,
    /// Number of complete passes.
    pub iterations: u32,
    /// Deterministic node visits and emissions.
    pub work_units: u64,
    /// Exact ordered rule applications for a successful fixed point.
    pub steps: Vec<RewriteStep>,
    /// Exact output formula identity only on success.
    pub output_identity: Option<String>,
    /// Exact remapped fairness identity only on success with fairness input.
    pub output_fairness_identity: Option<String>,
    /// A same-edition fixed-point formula only on success.
    pub output: Option<InfiniteFormulaDocument>,
    /// Same-graph remapped ordered fairness roots only on success.
    pub output_fairness: Option<FairnessPremisesDocument>,
}

impl InfiniteRewriteReport {
    /// Strict-reads a canonical report under the existing owner/caller bounds.
    pub fn from_json_bytes(bytes: &[u8], limits: RecordLimits) -> Result<Self, RecordReadError> {
        read_canonical(bytes, limits)
    }

    fn fail(&mut self, reason: InfiniteRewriteFailure, work: u64, iterations: u32) {
        self.status = match reason {
            InfiniteRewriteFailure::IdentityMismatch
            | InfiniteRewriteFailure::UnmappableFairness => RewriteStatus::InvalidInput,
            InfiniteRewriteFailure::Budget(_) | InfiniteRewriteFailure::ReportBytes => {
                RewriteStatus::BudgetExhausted
            }
            InfiniteRewriteFailure::Internal => RewriteStatus::Failed,
            InfiniteRewriteFailure::NonConvergent => RewriteStatus::NonConvergent,
        };
        self.failure = Some(reason);
        self.work_units = work;
        self.iterations = iterations;
        self.steps.clear();
        self.output_identity = None;
        self.output_fairness_identity = None;
        self.output = None;
        self.output_fairness = None;
    }

    /// Whether this attempt produced a complete, fixed-point graph.
    pub fn succeeded(&self) -> bool {
        matches!(
            self.status,
            RewriteStatus::Unchanged | RewriteStatus::Normalized
        ) && self.failure.is_none()
            && self.output.is_some()
    }
}

struct Pass {
    nodes: Vec<InfiniteNode>,
    interner: BTreeMap<Kind, NodeId>,
    work: u64,
    limits: RewriteBudgets,
}

impl Pass {
    fn charge(&mut self) -> Result<(), InfiniteRewriteFailure> {
        if self.work >= self.limits.max_work_units {
            return Err(InfiniteRewriteFailure::Budget(BudgetKind::WorkUnits));
        }
        self.work += 1;
        Ok(())
    }

    fn emit(
        &mut self,
        kind: Kind,
        span: Option<SourceSpan>,
    ) -> Result<NodeId, InfiniteRewriteFailure> {
        self.charge()?;
        if let Some(existing) = self.interner.get(&kind) {
            return Ok(*existing);
        }
        if self.nodes.len() >= usize::try_from(self.limits.max_nodes).unwrap_or(usize::MAX) {
            return Err(InfiniteRewriteFailure::Budget(BudgetKind::Nodes));
        }
        let id = NodeId(
            u32::try_from(self.nodes.len())
                .map_err(|_| InfiniteRewriteFailure::Budget(BudgetKind::Nodes))?,
        );
        self.nodes.push(InfiniteNode { kind, span });
        self.interner.insert(kind, id);
        Ok(id)
    }

    fn kind(&self, id: NodeId) -> Option<Kind> {
        usize::try_from(id.0)
            .ok()
            .and_then(|index| self.nodes.get(index))
            .map(|node| node.kind)
    }

    fn constant(&self, id: NodeId, value: bool) -> bool {
        self.kind(id) == Some(if value { Kind::True } else { Kind::False })
    }

    fn apply(
        &mut self,
        kind: Kind,
        span: Option<SourceSpan>,
    ) -> Result<Option<(&'static str, NodeId)>, InfiniteRewriteFailure> {
        let direct = match kind {
            Kind::Not { operand } => match self.kind(operand) {
                Some(Kind::False) => Some(("bool.not.false", self.emit(Kind::True, span)?)),
                Some(Kind::True) => Some(("bool.not.true", self.emit(Kind::False, span)?)),
                Some(Kind::Not { operand }) => Some(("bool.not.double", operand)),
                _ => None,
            },
            Kind::And { left, right: _ } if self.constant(left, false) => {
                Some(("bool.and.false-left", left))
            }
            Kind::And { left: _, right } if self.constant(right, false) => {
                Some(("bool.and.false-right", right))
            }
            Kind::And { left, right } if self.constant(left, true) => {
                Some(("bool.and.true-left", right))
            }
            Kind::And { left, right } if self.constant(right, true) => {
                Some(("bool.and.true-right", left))
            }
            Kind::And { left, right } if left == right => Some(("bool.and.idempotent", left)),
            Kind::Or { left, right: _ } if self.constant(left, true) => {
                Some(("bool.or.true-left", left))
            }
            Kind::Or { left: _, right } if self.constant(right, true) => {
                Some(("bool.or.true-right", right))
            }
            Kind::Or { left, right } if self.constant(left, false) => {
                Some(("bool.or.false-left", right))
            }
            Kind::Or { left, right } if self.constant(right, false) => {
                Some(("bool.or.false-right", left))
            }
            Kind::Or { left, right } if left == right => Some(("bool.or.idempotent", left)),
            Kind::Implies { left, right: _ } if self.constant(left, false) => {
                Some(("bool.implies.false-left", self.emit(Kind::True, span)?))
            }
            Kind::Implies { left, right } if self.constant(left, true) => {
                Some(("bool.implies.true-left", right))
            }
            Kind::Implies { left: _, right } if self.constant(right, true) => {
                Some(("bool.implies.true-right", right))
            }
            Kind::Implies { left, right } if self.constant(right, false) => Some((
                "bool.implies.false-right",
                self.emit(Kind::Not { operand: left }, span)?,
            )),
            Kind::Implies { left, right } if left == right => {
                Some(("bool.implies.reflexive", self.emit(Kind::True, span)?))
            }
            Kind::Implies { left, right } => {
                let not_left = self.emit(Kind::Not { operand: left }, None)?;
                Some((
                    "bool.implies.eliminate",
                    self.emit(
                        Kind::Or {
                            left: not_left,
                            right,
                        },
                        span,
                    )?,
                ))
            }
            Kind::Equivalent { left, right } if left == right => {
                Some(("bool.equivalent.reflexive", self.emit(Kind::True, span)?))
            }
            Kind::Equivalent { left, right } if self.constant(left, true) => {
                Some(("bool.equivalent.true-left", right))
            }
            Kind::Equivalent { left, right } if self.constant(right, true) => {
                Some(("bool.equivalent.true-right", left))
            }
            Kind::Equivalent { left, right } if self.constant(left, false) => Some((
                "bool.equivalent.false-left",
                self.emit(Kind::Not { operand: right }, span)?,
            )),
            Kind::Equivalent { left, right } if self.constant(right, false) => Some((
                "bool.equivalent.false-right",
                self.emit(Kind::Not { operand: left }, span)?,
            )),
            _ => None,
        };
        if direct.is_some() {
            return Ok(direct);
        }
        let temporal = match kind {
            Kind::Not { operand } => match self.kind(operand) {
                Some(Kind::Future { interval, operand }) => {
                    let negated = self.emit(Kind::Not { operand }, None)?;
                    Some((
                        "neg.future.dual",
                        self.emit(
                            Kind::Globally {
                                interval,
                                operand: negated,
                            },
                            span,
                        )?,
                    ))
                }
                Some(Kind::Globally { interval, operand }) => {
                    let negated = self.emit(Kind::Not { operand }, None)?;
                    Some((
                        "neg.globally.dual",
                        self.emit(
                            Kind::Future {
                                interval,
                                operand: negated,
                            },
                            span,
                        )?,
                    ))
                }
                Some(Kind::Until {
                    interval,
                    left,
                    right,
                }) => {
                    let left = self.emit(Kind::Not { operand: left }, None)?;
                    let right = self.emit(Kind::Not { operand: right }, None)?;
                    Some((
                        "neg.until.dual",
                        self.emit(
                            Kind::Release {
                                interval,
                                left,
                                right,
                            },
                            span,
                        )?,
                    ))
                }
                Some(Kind::Release {
                    interval,
                    left,
                    right,
                }) => {
                    let left = self.emit(Kind::Not { operand: left }, None)?;
                    let right = self.emit(Kind::Not { operand: right }, None)?;
                    Some((
                        "neg.release.dual",
                        self.emit(
                            Kind::Until {
                                interval,
                                left,
                                right,
                            },
                            span,
                        )?,
                    ))
                }
                Some(Kind::Since {
                    interval,
                    left,
                    right,
                }) => match (self.kind(left), self.kind(right)) {
                    (Some(Kind::Not { operand: p }), Some(Kind::Not { operand: q })) => Some((
                        "past.triggered.fold-dual",
                        self.emit(
                            Kind::Triggered {
                                interval,
                                left: p,
                                right: q,
                            },
                            span,
                        )?,
                    )),
                    _ => None,
                },
                _ => None,
            },
            Kind::Future { interval, operand } if singleton(interval) => {
                Some(("temporal.future.singleton", operand))
            }
            Kind::Future { operand, .. } if self.constant(operand, false) => {
                Some(("temporal.future.false", self.emit(Kind::False, span)?))
            }
            Kind::Future { operand, .. } if self.constant(operand, true) => {
                Some(("temporal.future.true", self.emit(Kind::True, span)?))
            }
            Kind::Globally { interval, operand } if singleton(interval) => {
                Some(("temporal.globally.singleton", operand))
            }
            Kind::Globally { operand, .. } if self.constant(operand, false) => {
                Some(("temporal.globally.false", self.emit(Kind::False, span)?))
            }
            Kind::Globally { operand, .. } if self.constant(operand, true) => {
                Some(("temporal.globally.true", self.emit(Kind::True, span)?))
            }
            Kind::Until {
                interval, right, ..
            } if singleton(interval) => Some(("temporal.until.singleton", right)),
            Kind::Until {
                interval,
                left,
                right,
            } if self.constant(left, true) => Some((
                "temporal.until.true-left",
                self.emit(
                    Kind::Future {
                        interval,
                        operand: right,
                    },
                    span,
                )?,
            )),
            Kind::Release {
                interval, right, ..
            } if singleton(interval) => Some(("temporal.release.singleton", right)),
            Kind::Release {
                interval,
                left,
                right,
            } if self.constant(left, false) => Some((
                "temporal.release.false-left",
                self.emit(
                    Kind::Globally {
                        interval,
                        operand: right,
                    },
                    span,
                )?,
            )),
            Kind::Once { interval, operand } if previous(interval) => Some((
                "past.once.strong-previous",
                self.emit(Kind::StrongPrevious { operand }, span)?,
            )),
            Kind::False
            | Kind::True
            | Kind::Proposition { .. }
            | Kind::And { .. }
            | Kind::Or { .. }
            | Kind::Implies { .. }
            | Kind::Equivalent { .. }
            | Kind::Future { .. }
            | Kind::Globally { .. }
            | Kind::Until { .. }
            | Kind::Release { .. }
            | Kind::Once { .. }
            | Kind::Historically { .. }
            | Kind::StrongPrevious { .. }
            | Kind::Since { .. }
            | Kind::Triggered { .. } => None,
        };
        Ok(temporal)
    }
}

fn singleton(interval: TemporalInterval) -> bool {
    matches!(interval, TemporalInterval::Closed(range) if range.start() == 0 && range.end() == 0)
}

fn previous(interval: TemporalInterval) -> bool {
    matches!(interval, TemporalInterval::Closed(range) if range.start() == 1 && range.end() == 1)
}

fn remap(kind: Kind, mapped: &[NodeId]) -> Option<Kind> {
    let id = |old: NodeId| {
        usize::try_from(old.0)
            .ok()
            .and_then(|index| mapped.get(index))
            .copied()
    };
    Some(match kind {
        Kind::False => Kind::False,
        Kind::True => Kind::True,
        Kind::Proposition { proposition } => Kind::Proposition { proposition },
        Kind::Not { operand } => Kind::Not {
            operand: id(operand)?,
        },
        Kind::And { left, right } => Kind::And {
            left: id(left)?,
            right: id(right)?,
        },
        Kind::Or { left, right } => Kind::Or {
            left: id(left)?,
            right: id(right)?,
        },
        Kind::Implies { left, right } => Kind::Implies {
            left: id(left)?,
            right: id(right)?,
        },
        Kind::Equivalent { left, right } => Kind::Equivalent {
            left: id(left)?,
            right: id(right)?,
        },
        Kind::Future { interval, operand } => Kind::Future {
            interval,
            operand: id(operand)?,
        },
        Kind::Globally { interval, operand } => Kind::Globally {
            interval,
            operand: id(operand)?,
        },
        Kind::Until {
            interval,
            left,
            right,
        } => Kind::Until {
            interval,
            left: id(left)?,
            right: id(right)?,
        },
        Kind::Release {
            interval,
            left,
            right,
        } => Kind::Release {
            interval,
            left: id(left)?,
            right: id(right)?,
        },
        Kind::Once { interval, operand } => Kind::Once {
            interval,
            operand: id(operand)?,
        },
        Kind::Historically { interval, operand } => Kind::Historically {
            interval,
            operand: id(operand)?,
        },
        Kind::StrongPrevious { operand } => Kind::StrongPrevious {
            operand: id(operand)?,
        },
        Kind::Since {
            interval,
            left,
            right,
        } => Kind::Since {
            interval,
            left: id(left)?,
            right: id(right)?,
        },
        Kind::Triggered {
            interval,
            left,
            right,
        } => Kind::Triggered {
            interval,
            left: id(left)?,
            right: id(right)?,
        },
    })
}

fn compact(
    input: &InfiniteFormulaDocument,
    nodes: &[InfiniteNode],
    root: NodeId,
    fairness: &[NodeId],
) -> Result<(InfiniteFormulaDocument, Vec<NodeId>), InfiniteRewriteFailure> {
    let mut reachable = vec![false; nodes.len()];
    let mut pending = Vec::with_capacity(fairness.len() + 1);
    pending.push(root);
    pending.extend_from_slice(fairness);
    while let Some(id) = pending.pop() {
        let index = usize::try_from(id.0).map_err(|_| InfiniteRewriteFailure::Internal)?;
        let present = reachable
            .get_mut(index)
            .ok_or(InfiniteRewriteFailure::Internal)?;
        if *present {
            continue;
        }
        *present = true;
        pending.extend(nodes[index].kind.operands().into_iter().flatten());
    }
    let mut mapped = vec![NodeId(0); nodes.len()];
    let mut compacted = Vec::new();
    for (index, node) in nodes.iter().enumerate() {
        if !reachable[index] {
            continue;
        }
        let kind = remap(node.kind, &mapped).ok_or(InfiniteRewriteFailure::Internal)?;
        mapped[index] = NodeId(
            u32::try_from(compacted.len())
                .map_err(|_| InfiniteRewriteFailure::Budget(BudgetKind::Nodes))?,
        );
        compacted.push(InfiniteNode {
            kind,
            span: node.span,
        });
    }
    let map_root = |id: NodeId| -> Result<NodeId, InfiniteRewriteFailure> {
        usize::try_from(id.0)
            .ok()
            .and_then(|index| mapped.get(index))
            .copied()
            .ok_or(InfiniteRewriteFailure::Internal)
    };
    let root = map_root(root)?;
    let fairness = fairness
        .iter()
        .copied()
        .map(map_root)
        .collect::<Result<Vec<_>, _>>()?;
    let output =
        InfiniteFormulaDocument::new(input.semantic_profile(), input.clock(), root, compacted)
            .map_err(|_| InfiniteRewriteFailure::Internal)?;
    Ok((output, fairness))
}

fn base(
    input: &InfiniteFormulaDocument,
    fairness: Option<&FairnessPremisesDocument>,
    options: RewriteOptions,
    source_revision: &str,
    max_report_bytes: usize,
) -> InfiniteRewriteReport {
    let input_identity = input.content_identity().unwrap_or_default();
    let input_fairness_identity = fairness.and_then(|value| value.content_identity().ok());
    let catalog_sha256 = infinite_catalog().catalog_sha256;
    let max_report_bytes = max_report_bytes.min(MAX_INFINITE_REPORT_BYTES);
    let request_sha256 = sha256_json(&(
        "tl-rewrite.infinite-request/v1",
        &input_identity,
        &input_fairness_identity,
        options,
        source_revision,
        &catalog_sha256,
        max_report_bytes,
    ));
    InfiniteRewriteReport {
        schema_version: "tl-rewrite.infinite-report/v1".to_owned(),
        input_identity,
        input_fairness_identity,
        source_revision: source_revision.to_owned(),
        catalog_sha256,
        request_sha256,
        options,
        max_report_bytes,
        status: RewriteStatus::InvalidInput,
        failure: None,
        iterations: 0,
        work_units: 0,
        steps: Vec::new(),
        output_identity: None,
        output_fairness_identity: None,
        output: None,
        output_fairness: None,
    }
}

/// Rewrites a validated infinite graph to a bounded fixed point.
///
/// No failed attempt exposes a partial graph, fairness set, or ordered trace.
pub fn rewrite_infinite(
    input: &InfiniteFormulaDocument,
    fairness: Option<&FairnessPremisesDocument>,
    options: RewriteOptions,
    source_revision: &str,
    max_report_bytes: usize,
) -> InfiniteRewriteReport {
    let mut report = base(input, fairness, options, source_revision, max_report_bytes);
    if report.input_identity.is_empty()
        || source_revision.is_empty()
        || fairness.is_some_and(|premises| {
            premises.graph_identity() != report.input_identity
                || premises.clock() != input.clock()
                || report.input_fairness_identity.is_none()
        })
    {
        report.fail(InfiniteRewriteFailure::IdentityMismatch, 0, 0);
        return report;
    }
    let mut current = input.clone();
    let mut roots = fairness.map_or_else(Vec::new, |premises| premises.roots().to_vec());
    let mut seen = BTreeSet::from([report.input_identity.clone()]);
    let mut all_steps = Vec::new();
    let mut rolling = sha256_bytes(b"tl-rewrite.infinite-trace/v1");
    let mut state = Pass {
        nodes: Vec::new(),
        interner: BTreeMap::new(),
        work: 0,
        limits: options.budgets,
    };
    for iteration in 0..options.budgets.max_iterations {
        state.nodes.clear();
        state.interner.clear();
        let mut mapped = Vec::with_capacity(current.nodes().len());
        for (index, node) in current.nodes().iter().enumerate() {
            if let Err(reason) = state.charge() {
                report.fail(reason, state.work, iteration);
                return report;
            }
            let Some(kind) = remap(node.kind, &mapped) else {
                report.fail(InfiniteRewriteFailure::Internal, state.work, iteration);
                return report;
            };
            let application = match state.apply(kind, node.span) {
                Ok(value) => value,
                Err(reason) => {
                    report.fail(reason, state.work, iteration);
                    return report;
                }
            };
            let output = if let Some((rule_id, replacement)) = application {
                if u64::try_from(all_steps.len()).unwrap_or(u64::MAX)
                    >= options.budgets.max_rule_applications
                {
                    report.fail(
                        InfiniteRewriteFailure::Budget(BudgetKind::RuleApplications),
                        state.work,
                        iteration,
                    );
                    return report;
                }
                let Some(after) = state.kind(replacement) else {
                    report.fail(InfiniteRewriteFailure::Internal, state.work, iteration);
                    return report;
                };
                let before_sha256 = sha256_json(&kind);
                let after_sha256 = sha256_json(&after);
                rolling = sha256_bytes(
                    format!("{rolling}\0{rule_id}\01\0{before_sha256}\0{after_sha256}").as_bytes(),
                );
                all_steps.push(RewriteStep {
                    sequence: u64::try_from(all_steps.len()).unwrap_or(u64::MAX),
                    pass: iteration,
                    source_node: u32::try_from(index).unwrap_or(u32::MAX),
                    source_span: node.span,
                    rule_id: rule_id.to_owned(),
                    rule_revision: 1,
                    before_sha256,
                    after_sha256,
                    intermediate_sha256: rolling.clone(),
                });
                replacement
            } else {
                match state.emit(kind, node.span) {
                    Ok(id) => id,
                    Err(reason) => {
                        report.fail(reason, state.work, iteration);
                        return report;
                    }
                }
            };
            mapped.push(output);
        }
        let map = |id: NodeId| -> Option<NodeId> {
            usize::try_from(id.0)
                .ok()
                .and_then(|index| mapped.get(index))
                .copied()
        };
        let Some(root) = map(current.root()) else {
            report.fail(InfiniteRewriteFailure::Internal, state.work, iteration);
            return report;
        };
        let Some(next_roots) = roots.iter().copied().map(map).collect::<Option<Vec<_>>>() else {
            report.fail(InfiniteRewriteFailure::Internal, state.work, iteration);
            return report;
        };
        let (candidate, candidate_roots) = match compact(&current, &state.nodes, root, &next_roots)
        {
            Ok(value) => value,
            Err(reason) => {
                report.fail(reason, state.work, iteration);
                return report;
            }
        };
        let candidate_identity = match candidate.content_identity() {
            Ok(value) => value,
            Err(_) => {
                report.fail(InfiniteRewriteFailure::Internal, state.work, iteration);
                return report;
            }
        };
        let candidate_fairness = if fairness.is_some() {
            match FairnessPremisesDocument::new(
                &candidate,
                candidate_identity.clone(),
                candidate.clock(),
                candidate_roots.clone(),
            ) {
                Ok(value) => Some(value),
                Err(_) => {
                    report.fail(
                        InfiniteRewriteFailure::UnmappableFairness,
                        state.work,
                        iteration,
                    );
                    return report;
                }
            }
        } else {
            None
        };
        let stable = candidate == current && candidate_roots == roots;
        report.iterations = iteration + 1;
        if stable {
            report.status = if all_steps.is_empty() && candidate == *input {
                RewriteStatus::Unchanged
            } else {
                RewriteStatus::Normalized
            };
            report.failure = None;
            report.work_units = state.work;
            report.steps = all_steps;
            report.output_identity = Some(candidate_identity);
            report.output_fairness_identity = match candidate_fairness.as_ref() {
                Some(value) => match value.content_identity() {
                    Ok(identity) => Some(identity),
                    Err(_) => {
                        report.fail(InfiniteRewriteFailure::Internal, state.work, iteration + 1);
                        return report;
                    }
                },
                None => None,
            };
            report.output = Some(candidate);
            report.output_fairness = candidate_fairness;
            let encoded = match serde_json::to_vec(&report) {
                Ok(bytes) => bytes,
                Err(_) => {
                    report.fail(InfiniteRewriteFailure::Internal, state.work, iteration + 1);
                    return report;
                }
            };
            if encoded.len() > report.max_report_bytes {
                report.fail(
                    InfiniteRewriteFailure::ReportBytes,
                    state.work,
                    iteration + 1,
                );
            }
            return report;
        }
        if !seen.insert(candidate_identity) {
            report.fail(
                InfiniteRewriteFailure::NonConvergent,
                state.work,
                iteration + 1,
            );
            return report;
        }
        current = candidate;
        roots = candidate_roots;
    }
    report.fail(
        InfiniteRewriteFailure::Budget(BudgetKind::Iterations),
        state.work,
        options.budgets.max_iterations,
    );
    report
}

/// Reexecutes an exact infinite report under the caller's trusted source revision.
/// A mismatch returns false and carries no proof or soundness credit.
pub fn replay_infinite(
    input: &InfiniteFormulaDocument,
    fairness: Option<&FairnessPremisesDocument>,
    expected: &InfiniteRewriteReport,
    trusted_source_revision: &str,
) -> bool {
    if !expected.succeeded()
        || expected.source_revision != trusted_source_revision
        || expected.catalog_sha256 != infinite_catalog().catalog_sha256
    {
        return false;
    }
    let reproduced = rewrite_infinite(
        input,
        fairness,
        expected.options,
        trusted_source_revision,
        expected.max_report_bytes,
    );
    match (
        serde_json::to_vec(&reproduced),
        serde_json::to_vec(expected),
    ) {
        (Ok(actual), Ok(record)) => record.len() <= expected.max_report_bytes && actual == record,
        _ => false,
    }
}

/// Outcome of a trace-scoped provider comparison for one completed rewrite.
#[cfg(feature = "infinite-trace")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InfiniteConformanceStatus {
    /// Both completed provider results have the same semantic axes.
    Equivalent,
    /// Completed provider results differ on a semantic axis.
    Mismatch,
    /// The comparison could not make a semantic claim.
    NonConclusive,
}

/// Typed reason a provider comparison has no semantic claim.
#[cfg(feature = "infinite-trace")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InfiniteConformanceReason {
    /// The rewrite record cannot be replayed against the trusted inputs.
    InvalidRewrite,
    /// The trace and formula have incompatible profile or clock identities.
    InvalidTrace,
    /// Conflicting evidence was present before either provider evaluation.
    ConflictingObservation,
    /// Neither graph had any completion admitted by fairness.
    EmptyFairAdmission,
    /// A configured provider ceiling prevented complete evaluation.
    ResourceIncomplete,
    /// A provider refused malformed or mismatched input.
    ProviderRefusal,
    /// A provider returned an unsuccessful execution disposition.
    ProviderFailure,
}

/// Exact provider evidence for a trace-scoped comparison, never a model proof.
#[cfg(feature = "infinite-trace")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InfiniteConformanceReport {
    /// Comparison state.
    pub status: InfiniteConformanceStatus,
    /// Typed nonconclusive reason, absent on completed comparisons.
    pub reason: Option<InfiniteConformanceReason>,
    /// Original graph's provider result, absent when preclassification refused.
    pub before: Option<tl_mltl::infinite::InfiniteResult>,
    /// Rewritten graph's provider result, absent when preclassification refused.
    pub after: Option<tl_mltl::infinite::InfiniteResult>,
}

/// Compares the original and rewritten formula on the same exact lasso.
///
/// This is differential, trace-scoped evidence. Independent soundness
/// qualification remains the responsibility of the dev-only oracle.
#[cfg(feature = "infinite-trace")]
pub fn check_infinite_rewrite(
    input: &InfiniteFormulaDocument,
    fairness: Option<&FairnessPremisesDocument>,
    trace: &tl_syntax::LassoTraceDocument,
    report: &InfiniteRewriteReport,
    selected_position: u64,
    trusted_source_revision: &str,
    limit: tl_mltl::infinite::EvaluationLimit,
) -> InfiniteConformanceReport {
    use tl_mltl::infinite::{evaluate_lasso, EvidenceClosure, InfiniteError, LassoRequest};

    let refuse = |reason| InfiniteConformanceReport {
        status: InfiniteConformanceStatus::NonConclusive,
        reason: Some(reason),
        before: None,
        after: None,
    };
    if !replay_infinite(input, fairness, report, trusted_source_revision) {
        return refuse(InfiniteConformanceReason::InvalidRewrite);
    }
    if trace.semantic_profile() != input.semantic_profile() || trace.clock() != input.clock() {
        return refuse(InfiniteConformanceReason::InvalidTrace);
    }
    // Classify the whole admitted trace before evaluating either formula: a
    // Boolean fold may otherwise hide conflicting evidence in a dead branch.
    if trace
        .prefix()
        .iter()
        .chain(trace.loop_observations())
        .flat_map(|observation| observation.valuation.entries())
        .any(|entry| entry.value == tl_syntax::PartialValue::Conflicting)
    {
        return refuse(InfiniteConformanceReason::ConflictingObservation);
    }
    let Some(output) = report.output.as_ref() else {
        return refuse(InfiniteConformanceReason::InvalidRewrite);
    };
    let Some(output_identity) = report.output_identity.as_deref() else {
        return refuse(InfiniteConformanceReason::InvalidRewrite);
    };
    let Ok(trace_id) = trace.content_identity() else {
        return refuse(InfiniteConformanceReason::InvalidTrace);
    };
    let original = LassoRequest {
        formula: input,
        trace,
        fairness,
        evidence_closure: EvidenceClosure::Closed,
        graph_id: &report.input_identity,
        trace_id: &trace_id,
        selected_position,
        limit,
    };
    let rewritten = LassoRequest {
        formula: output,
        trace,
        fairness: report.output_fairness.as_ref(),
        evidence_closure: EvidenceClosure::Closed,
        graph_id: output_identity,
        trace_id: &trace_id,
        selected_position,
        limit,
    };
    let map_error = |error| match error {
        InfiniteError::ResourceIncomplete => InfiniteConformanceReason::ResourceIncomplete,
        InfiniteError::InvalidFormula
        | InfiniteError::InvalidLasso
        | InfiniteError::IdentityMismatch => InfiniteConformanceReason::ProviderRefusal,
    };
    let before = match evaluate_lasso(&original) {
        Ok(value) => value,
        Err(error) => return refuse(map_error(error)),
    };
    let after = match evaluate_lasso(&rewritten) {
        Ok(value) => value,
        Err(error) => {
            return InfiniteConformanceReport {
                status: InfiniteConformanceStatus::NonConclusive,
                reason: Some(map_error(error)),
                before: Some(before),
                after: None,
            };
        }
    };
    let (status, nonconclusive) = classify_infinite_results(&before, &after);
    InfiniteConformanceReport {
        status,
        reason: nonconclusive,
        before: Some(before),
        after: Some(after),
    }
}

/// Classifies two completed provider results without claiming model-wide
/// equivalence. Keeping this decision separate permits each typed result axis
/// to be checked even when a sound rewrite normally yields equal outcomes.
#[cfg(feature = "infinite-trace")]
fn classify_infinite_results(
    before: &tl_mltl::infinite::InfiniteResult,
    after: &tl_mltl::infinite::InfiniteResult,
) -> (InfiniteConformanceStatus, Option<InfiniteConformanceReason>) {
    use tl_mltl::infinite::{Disposition, ExecutionDisposition, ResultReason};

    let nonconclusive = if before.disposition == Disposition::Unsupported
        || after.disposition == Disposition::Unsupported
    {
        Some(InfiniteConformanceReason::ProviderRefusal)
    } else if before.disposition == Disposition::Failed
        || after.disposition == Disposition::Failed
        || before.execution != ExecutionDisposition::Completed
        || after.execution != ExecutionDisposition::Completed
    {
        Some(
            if before.reason == Some(ResultReason::ResourceIncomplete)
                || after.reason == Some(ResultReason::ResourceIncomplete)
            {
                InfiniteConformanceReason::ResourceIncomplete
            } else {
                InfiniteConformanceReason::ProviderFailure
            },
        )
    } else if before.disposition == Disposition::Inconclusive
        && before.reason == Some(ResultReason::EmptyFairAdmission)
        || after.disposition == Disposition::Inconclusive
            && after.reason == Some(ResultReason::EmptyFairAdmission)
    {
        Some(InfiniteConformanceReason::EmptyFairAdmission)
    } else {
        None
    };
    let status = if nonconclusive.is_some() {
        InfiniteConformanceStatus::NonConclusive
    } else if before.disposition == after.disposition
        && before.execution == after.execution
        && before.truth == after.truth
        && before.basis == after.basis
        && before.reason == after.reason
        && before.admitted_completions == after.admitted_completions
    {
        InfiniteConformanceStatus::Equivalent
    } else {
        InfiniteConformanceStatus::Mismatch
    };
    (status, nonconclusive)
}

#[cfg(all(test, feature = "infinite-trace"))]
mod conformance_classification_tests {
    use super::{classify_infinite_results, InfiniteConformanceReason, InfiniteConformanceStatus};
    use tl_mltl::infinite::{
        Disposition, EvidenceBasis, EvidenceClosure, ExecutionDisposition, InfiniteResult,
        ResultIdentity, ResultReason, SubjectKind, TruthAvailability,
    };

    fn completed_result() -> InfiniteResult {
        InfiniteResult {
            disposition: Disposition::Proved,
            execution: ExecutionDisposition::Completed,
            truth: TruthAvailability::True,
            basis: EvidenceBasis::ExactTrace,
            reason: None,
            uncertainty: None,
            evidence: None,
            identity: ResultIdentity {
                feature: "infinite-trace",
                provider_revision: "fixture",
                profile: "mltl.infinite-trace/v1",
                graph_id: "graph".to_owned(),
                proposition_map_id: "map".to_owned(),
                subject_kind: SubjectKind::Lasso,
                subject_id: "trace".to_owned(),
                trace_id: Some("trace".to_owned()),
                clock: "event_position",
                selected_position: 0,
                fairness: vec![],
                evidence_closure: Some(EvidenceClosure::Closed),
            },
            admitted_completions: 1,
            evaluation_steps: 1,
        }
    }

    // Trace: TC-184, FR-047-AC-2; TC-074, FR-021-AC-2
    #[test]
    fn each_nonconclusive_result_axis_has_its_own_typed_reason() {
        let baseline = completed_result();
        for change_before in [true, false] {
            let (mut before, mut after) = (baseline.clone(), baseline.clone());
            let changed = if change_before {
                &mut before
            } else {
                &mut after
            };
            changed.disposition = Disposition::Unsupported;
            assert_eq!(
                classify_infinite_results(&before, &after),
                (
                    InfiniteConformanceStatus::NonConclusive,
                    Some(InfiniteConformanceReason::ProviderRefusal)
                )
            );

            let (mut before, mut after) = (baseline.clone(), baseline.clone());
            let changed = if change_before {
                &mut before
            } else {
                &mut after
            };
            changed.disposition = Disposition::Failed;
            assert_eq!(
                classify_infinite_results(&before, &after),
                (
                    InfiniteConformanceStatus::NonConclusive,
                    Some(InfiniteConformanceReason::ProviderFailure)
                )
            );

            let (mut before, mut after) = (baseline.clone(), baseline.clone());
            let changed = if change_before {
                &mut before
            } else {
                &mut after
            };
            changed.execution = ExecutionDisposition::Failed;
            assert_eq!(
                classify_infinite_results(&before, &after),
                (
                    InfiniteConformanceStatus::NonConclusive,
                    Some(InfiniteConformanceReason::ProviderFailure)
                )
            );

            let (mut before, mut after) = (baseline.clone(), baseline.clone());
            let changed = if change_before {
                &mut before
            } else {
                &mut after
            };
            changed.execution = ExecutionDisposition::ResourceIncomplete;
            changed.reason = Some(ResultReason::ResourceIncomplete);
            assert_eq!(
                classify_infinite_results(&before, &after),
                (
                    InfiniteConformanceStatus::NonConclusive,
                    Some(InfiniteConformanceReason::ResourceIncomplete)
                )
            );

            let (mut before, mut after) = (baseline.clone(), baseline.clone());
            let changed = if change_before {
                &mut before
            } else {
                &mut after
            };
            changed.disposition = Disposition::Inconclusive;
            changed.reason = Some(ResultReason::EmptyFairAdmission);
            assert_eq!(
                classify_infinite_results(&before, &after),
                (
                    InfiniteConformanceStatus::NonConclusive,
                    Some(InfiniteConformanceReason::EmptyFairAdmission)
                )
            );
        }
    }

    // Trace: TC-184, FR-047-AC-2; TC-069, FR-019-AC-3
    #[test]
    fn every_compared_result_axis_can_prevent_false_equivalence() {
        let before = completed_result();
        assert_eq!(
            classify_infinite_results(&before, &before),
            (InfiniteConformanceStatus::Equivalent, None)
        );
        let mut alternatives = Vec::new();
        let mut disposition = before.clone();
        disposition.disposition = Disposition::Refuted;
        alternatives.push(disposition);
        let mut truth = before.clone();
        truth.truth = TruthAvailability::False;
        alternatives.push(truth);
        let mut basis = before.clone();
        basis.basis = EvidenceBasis::BadPrefix;
        alternatives.push(basis);
        let mut reason = before.clone();
        reason.reason = Some(ResultReason::MissingObservation);
        alternatives.push(reason);
        let mut count = before.clone();
        count.admitted_completions = 2;
        alternatives.push(count);
        for after in alternatives {
            assert_eq!(
                classify_infinite_results(&before, &after),
                (InfiniteConformanceStatus::Mismatch, None)
            );
        }
    }
}
