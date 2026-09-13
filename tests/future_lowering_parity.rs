//! Direct-versus-lowered controls for the tl-syntax W/M lowering (FR-008).
//!
//! `W[a,b](p,q)` and `M[a,b](p,q)` exist only as tl-syntax lowering. The engine
//! consumes the primitive F/G/U/R and Boolean graph they lower to, so every
//! rewrite outcome of a lowered graph must equal the outcome of the same graph
//! built by hand. The lowering crates are dev-only and renamed; a lowered graph
//! reaches the production `tl_syntax` type only through the formula v1 wire.

mod common;

use std::{fs, path::PathBuf};

use common::document;
use tl_rewrite::{
    check_equivalence, rewrite, rewrite_with_context, BudgetKind, ConformanceOptions,
    ConformanceReport, ConformanceStatus, RewriteBudgets, RewriteOptions, RewriteReport,
    RewriteStatus,
};
use tl_syntax::{
    FormulaDocument, Interval, Node, NodeId, NodeKind, OwnedSignalDeclaration, PropositionBinding,
    PropositionId, SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId,
};
use tl_syntax_lowering as lowering;

const FORMULA_ID: &str = "future-lowering-parity";
const SOURCE_REVISION: &str = "source";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    W,
    M,
}

impl Kind {
    const fn spelling(self) -> &'static [u8] {
        match self {
            Self::W => b"W",
            Self::M => b"M",
        }
    }
}

/// A formula over the two derived kinds, emitted in post-order by both builders.
#[derive(Clone, Debug)]
enum Expr {
    Prop(u32),
    True,
    False,
    Not(Box<Expr>),
    /// `left kind[start,end] right`, each operand emitted separately.
    Derived(Kind, u32, u32, Box<Expr>, Box<Expr>),
    /// `operand kind[start,end] operand`, the one operand node shared by both sides.
    Shared(Kind, u32, u32, Box<Expr>),
}

fn prop(id: u32) -> Box<Expr> {
    Box::new(Expr::Prop(id))
}

fn derived(kind: Kind, start: u32, end: u32, left: Box<Expr>, right: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Derived(kind, start, end, left, right))
}

/// A wrong lowering, applied to the lowered side only.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mutation {
    None,
    /// Until and Release exchanged.
    SwapUntilRelease,
    /// Or and And exchanged.
    SwapOrAnd,
    /// Globally and Future exchanged.
    SwapGloballyFuture,
    /// The Globally/Future node quantifies the right operand.
    UnaryOverRight,
    /// The unary node's inclusive end bound is one past the binary node's.
    WidenedEndpoint,
    /// The lowered document claims the online-prefix profile.
    ChangedProfile,
    /// The unary node is generated before the binary node: same semantics, other graph.
    ReorderedGenerated,
    /// An extra `And(root, root)` node: the same semantics at a different resource charge.
    ExtraCharge,
    /// Generated nodes carry a source span: the same semantic identity, other report bytes.
    SpanAttribution,
}

/// Mutants that change the formula's meaning on at least one corpus case.
const SEMANTIC_MUTATIONS: [Mutation; 5] = [
    Mutation::SwapUntilRelease,
    Mutation::SwapOrAnd,
    Mutation::SwapGloballyFuture,
    Mutation::UnaryOverRight,
    Mutation::WidenedEndpoint,
];

/// Mutants that keep the meaning but change the graph or resource identity.
const SHAPE_MUTATIONS: [Mutation; 2] = [Mutation::ReorderedGenerated, Mutation::ExtraCharge];

const ALL_MUTATIONS: [Mutation; 9] = [
    Mutation::SwapUntilRelease,
    Mutation::SwapOrAnd,
    Mutation::SwapGloballyFuture,
    Mutation::UnaryOverRight,
    Mutation::WidenedEndpoint,
    Mutation::ChangedProfile,
    Mutation::ReorderedGenerated,
    Mutation::ExtraCharge,
    Mutation::SpanAttribution,
];

fn interval(start: u32, end: u32) -> Interval {
    Interval::new(start, end).unwrap()
}

fn node_id(index: usize) -> NodeId {
    NodeId(u32::try_from(index).unwrap())
}

/// Builds the primitive graph by hand in the production `tl_syntax` model.
fn direct(expr: &Expr, profile: SemanticProfile) -> FormulaDocument {
    fn emit(expr: &Expr, nodes: &mut Vec<Node>) -> NodeId {
        let kind = match expr {
            Expr::Prop(id) => NodeKind::Proposition {
                proposition: PropositionId(*id),
            },
            Expr::True => NodeKind::True,
            Expr::False => NodeKind::False,
            Expr::Not(operand) => NodeKind::Not {
                operand: emit(operand, nodes),
            },
            Expr::Derived(kind, start, end, left, right) => {
                let left = emit(left, nodes);
                let right = emit(right, nodes);
                return expand(*kind, interval(*start, *end), left, right, nodes);
            }
            Expr::Shared(kind, start, end, operand) => {
                let operand = emit(operand, nodes);
                return expand(*kind, interval(*start, *end), operand, operand, nodes);
            }
        };
        nodes.push(Node::new(kind));
        node_id(nodes.len() - 1)
    }

    fn expand(
        kind: Kind,
        interval: Interval,
        left: NodeId,
        right: NodeId,
        nodes: &mut Vec<Node>,
    ) -> NodeId {
        let binary = node_id(nodes.len());
        let unary = node_id(nodes.len() + 1);
        let (binary_kind, unary_kind, root_kind) = match kind {
            Kind::W => (
                NodeKind::Until {
                    interval,
                    left,
                    right,
                },
                NodeKind::Globally {
                    interval,
                    operand: left,
                },
                NodeKind::Or {
                    left: binary,
                    right: unary,
                },
            ),
            Kind::M => (
                NodeKind::Release {
                    interval,
                    left,
                    right,
                },
                NodeKind::Future {
                    interval,
                    operand: left,
                },
                NodeKind::And {
                    left: binary,
                    right: unary,
                },
            ),
        };
        nodes.extend([
            Node::new(binary_kind),
            Node::new(unary_kind),
            Node::new(root_kind),
        ]);
        node_id(nodes.len() - 1)
    }

    let mut nodes = Vec::new();
    emit(expr, &mut nodes);
    document(profile, nodes)
}

fn lowering_profile(profile: SemanticProfile) -> lowering::SemanticProfile {
    match profile {
        SemanticProfile::ClosedTraceV1 => lowering::SemanticProfile::ClosedTraceV1,
        SemanticProfile::OnlinePrefixV1 => lowering::SemanticProfile::OnlinePrefixV1,
    }
}

fn mutate(nodes: &mut [lowering::Node; 3], mutation: Mutation) {
    use lowering::NodeKind as K;

    let [binary, unary, root] = nodes;
    match mutation {
        Mutation::None | Mutation::ChangedProfile | Mutation::ExtraCharge => {}
        Mutation::SwapUntilRelease => {
            binary.kind = match binary.kind {
                K::Until {
                    interval,
                    left,
                    right,
                } => K::Release {
                    interval,
                    left,
                    right,
                },
                K::Release {
                    interval,
                    left,
                    right,
                } => K::Until {
                    interval,
                    left,
                    right,
                },
                other => panic!("unexpected lowered binary node {other:?}"),
            };
        }
        Mutation::SwapOrAnd => {
            root.kind = match root.kind {
                K::Or { left, right } => K::And { left, right },
                K::And { left, right } => K::Or { left, right },
                other => panic!("unexpected lowered root node {other:?}"),
            };
        }
        Mutation::SwapGloballyFuture => {
            unary.kind = match unary.kind {
                K::Globally { interval, operand } => K::Future { interval, operand },
                K::Future { interval, operand } => K::Globally { interval, operand },
                other => panic!("unexpected lowered unary node {other:?}"),
            };
        }
        Mutation::UnaryOverRight => {
            let right = match binary.kind {
                K::Until { right, .. } | K::Release { right, .. } => right,
                other => panic!("unexpected lowered binary node {other:?}"),
            };
            unary.kind = match unary.kind {
                K::Globally { interval, .. } => K::Globally {
                    interval,
                    operand: right,
                },
                K::Future { interval, .. } => K::Future {
                    interval,
                    operand: right,
                },
                other => panic!("unexpected lowered unary node {other:?}"),
            };
        }
        // Widening the binary node instead is a semantic no-op: a witness at `b+1`
        // already implies the unary disjunct (W) or is excluded by it (M).
        Mutation::WidenedEndpoint => match &mut unary.kind {
            K::Globally { interval, .. } | K::Future { interval, .. } => {
                *interval = lowering::Interval::new(interval.start(), interval.end() + 1).unwrap();
            }
            other => panic!("unexpected lowered unary node {other:?}"),
        },
        Mutation::ReorderedGenerated => {
            core::mem::swap(binary, unary);
            root.kind = match root.kind {
                K::Or { left, right } => K::Or {
                    left: right,
                    right: left,
                },
                K::And { left, right } => K::And {
                    left: right,
                    right: left,
                },
                other => panic!("unexpected lowered root node {other:?}"),
            };
        }
        Mutation::SpanAttribution => {
            let span = lowering::SourceSpan::new(0, 1).unwrap();
            for node in [binary, unary, root] {
                node.span = Some(span);
            }
        }
    }
}

/// Builds the graph through `FutureLoweringRequest::lower()` and carries it
/// across the formula v1 wire into the production `tl_syntax` model.
fn lowered(expr: &Expr, profile: SemanticProfile, mutation: Mutation) -> FormulaDocument {
    fn emit(
        expr: &Expr,
        profile: lowering::SemanticProfile,
        mutation: Mutation,
        nodes: &mut Vec<lowering::Node>,
    ) -> lowering::NodeId {
        use lowering::{NodeKind as K, PropositionId as P};

        let kind = match expr {
            Expr::Prop(id) => K::Proposition {
                proposition: P(*id),
            },
            Expr::True => K::True,
            Expr::False => K::False,
            Expr::Not(operand) => K::Not {
                operand: emit(operand, profile, mutation, nodes),
            },
            Expr::Derived(kind, start, end, left, right) => {
                let left = emit(left, profile, mutation, nodes);
                let right = emit(right, profile, mutation, nodes);
                return lower(*kind, (*start, *end), left, right, profile, mutation, nodes);
            }
            Expr::Shared(kind, start, end, operand) => {
                let operand = emit(operand, profile, mutation, nodes);
                return lower(
                    *kind,
                    (*start, *end),
                    operand,
                    operand,
                    profile,
                    mutation,
                    nodes,
                );
            }
        };
        nodes.push(lowering::Node::new(kind));
        lowering::NodeId(u32::try_from(nodes.len() - 1).unwrap())
    }

    fn lower(
        kind: Kind,
        (start, end): (u32, u32),
        left: lowering::NodeId,
        right: lowering::NodeId,
        profile: lowering::SemanticProfile,
        mutation: Mutation,
        nodes: &mut Vec<lowering::Node>,
    ) -> lowering::NodeId {
        let root = node_id_u32(nodes.len() - 1);
        let formula = lowering::Formula::new(profile, lowering::NodeId(root), nodes).unwrap();
        let expansion = lowering::FutureLoweringRequest {
            request_identity: lowering::FUTURE_LOWERING_REQUEST_V1.as_bytes(),
            operator_profile: lowering::FUTURE_OPERATORS_V1.as_bytes(),
            kind: kind.spelling(),
            semantic_profile: profile.as_str().as_bytes(),
            formula,
            left: u64::from(left.0),
            right: u64::from(right.0),
            interval: Some(lowering::RawBounds::new(u64::from(start), u64::from(end))),
            operator_span: None,
            expression_span: None,
        }
        .lower()
        .unwrap();
        assert_eq!(
            expansion.report().generated_count(),
            lowering::FUTURE_LOWERING_NODE_CHARGE
        );
        assert_eq!(expansion.report().semantic_profile(), profile);
        let mut generated = *expansion.nodes();
        mutate(&mut generated, mutation);
        nodes.extend(generated);
        assert_eq!(
            expansion.root(),
            lowering::NodeId(node_id_u32(nodes.len() - 1))
        );
        expansion.root()
    }

    fn node_id_u32(index: usize) -> u32 {
        u32::try_from(index).unwrap()
    }

    let wire_profile = if mutation == Mutation::ChangedProfile {
        lowering::SemanticProfile::OnlinePrefixV1
    } else {
        lowering_profile(profile)
    };
    let profile = lowering_profile(profile);
    let mut nodes = Vec::new();
    let mut root = emit(expr, profile, mutation, &mut nodes);
    if mutation == Mutation::ExtraCharge {
        nodes.push(lowering::Node::new(lowering::NodeKind::And {
            left: root,
            right: root,
        }));
        root = lowering::NodeId(node_id_u32(nodes.len() - 1));
    }
    let source = lowering::FormulaDocument::new(wire_profile, root, nodes).unwrap();
    wire(&source)
}

/// Serializes a lowering-lane document and decodes it as the production type.
fn wire(source: &lowering::FormulaDocument) -> FormulaDocument {
    let bytes = serde_json::to_vec(source).unwrap();
    let decoded: FormulaDocument = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        serde_json::to_vec(&decoded).unwrap(),
        bytes,
        "the formula v1 wire must round-trip byte for byte"
    );
    decoded
}

fn catalog(propositions: &[u32]) -> SignalCatalogDocument {
    let mut propositions = propositions.to_vec();
    propositions.sort_unstable();
    SignalCatalogDocument::new(
        propositions
            .iter()
            .map(|id| {
                OwnedSignalDeclaration::new(
                    SignalId(100 + id),
                    format!("signal_{id}"),
                    SignalDomain::Boolean,
                )
            })
            .collect(),
        propositions
            .iter()
            .map(|id| PropositionBinding::new(PropositionId(*id), SignalId(100 + id)))
            .collect(),
    )
    .unwrap()
}

fn budgets() -> [(BudgetKind, RewriteBudgets); 4] {
    let default = RewriteBudgets::default();
    [
        (
            BudgetKind::Iterations,
            RewriteBudgets {
                max_iterations: 1,
                ..default
            },
        ),
        (
            BudgetKind::Nodes,
            RewriteBudgets {
                max_nodes: 2,
                ..default
            },
        ),
        (
            BudgetKind::RuleApplications,
            RewriteBudgets {
                max_rule_applications: 0,
                ..default
            },
        ),
        (
            BudgetKind::WorkUnits,
            RewriteBudgets {
                max_work_units: 1,
                ..default
            },
        ),
    ]
}

fn options(budgets: RewriteBudgets) -> RewriteOptions {
    RewriteOptions {
        budgets,
        ..RewriteOptions::default()
    }
}

/// Every engine outcome observed for one input: rewrite reports under default
/// and tight budgets, contextual reports with a complete and an incomplete
/// catalog, and the exhaustive conformance report of the input against its output.
#[derive(Debug, Eq, PartialEq)]
struct Outcomes {
    default: RewriteReport,
    budgeted: Vec<RewriteReport>,
    bound: RewriteReport,
    unbound: RewriteReport,
    conformance: Option<ConformanceReport>,
}

fn outcomes(input: &FormulaDocument, propositions: &[u32]) -> Outcomes {
    let default = rewrite(
        input,
        FORMULA_ID,
        RewriteOptions::default(),
        SOURCE_REVISION,
    );
    let budgeted = budgets()
        .into_iter()
        .map(|(_, budgets)| rewrite(input, FORMULA_ID, options(budgets), SOURCE_REVISION))
        .collect();
    let bound = rewrite_with_context(
        input,
        FORMULA_ID,
        RewriteOptions::default(),
        SOURCE_REVISION,
        &catalog(propositions),
        None,
    );
    let unbound = rewrite_with_context(
        input,
        FORMULA_ID,
        RewriteOptions::default(),
        SOURCE_REVISION,
        &catalog(&propositions[1..]),
        None,
    );
    let conformance = default
        .output
        .as_ref()
        .map(|output| check_equivalence(input, output, FORMULA_ID, ConformanceOptions::default()));
    Outcomes {
        default,
        budgeted,
        bound,
        unbound,
        conformance,
    }
}

/// The parity comparator the mutation controls must be able to fail.
fn parity(
    direct: &FormulaDocument,
    lowered: &FormulaDocument,
    propositions: &[u32],
) -> Result<(), String> {
    if direct != lowered {
        return Err("the lowered graph differs from the direct graph".to_owned());
    }
    let direct = outcomes(direct, propositions);
    let lowered = outcomes(lowered, propositions);
    if direct.default != lowered.default {
        return Err(format!(
            "rewrite reports differ: {:?} / {:?}",
            direct.default.status, lowered.default.status
        ));
    }
    if direct.budgeted != lowered.budgeted {
        return Err("budgeted rewrite reports differ".to_owned());
    }
    if direct.bound != lowered.bound || direct.unbound != lowered.unbound {
        return Err("contextual rewrite reports differ".to_owned());
    }
    if direct.conformance != lowered.conformance {
        return Err("conformance reports differ".to_owned());
    }
    Ok(())
}

/// Like [`parity`] but without the up-front graph comparison, so a mutant must
/// be caught by the engine outcomes alone.
fn outcome_parity(
    direct: &FormulaDocument,
    lowered: &FormulaDocument,
    propositions: &[u32],
) -> Result<(), String> {
    if outcomes(direct, propositions) == outcomes(lowered, propositions) {
        Ok(())
    } else {
        Err("engine outcomes differ".to_owned())
    }
}

struct Case {
    name: &'static str,
    expr: Box<Expr>,
    propositions: &'static [u32],
}

fn cases() -> Vec<Case> {
    use Kind::{M, W};

    let mut cases = Vec::new();
    for kind in [W, M] {
        for (start, end) in [(0, 0), (0, 3), (2, 5)] {
            cases.push(Case {
                name: "distinct propositions",
                expr: derived(kind, start, end, prop(0), prop(1)),
                propositions: &[0, 1],
            });
        }
        cases.push(Case {
            name: "shared operand",
            expr: Box::new(Expr::Shared(kind, 1, 4, prop(0))),
            propositions: &[0],
        });
        cases.push(Case {
            name: "true left operand",
            expr: derived(kind, 0, 2, Box::new(Expr::True), prop(1)),
            propositions: &[1],
        });
        cases.push(Case {
            name: "false left operand",
            expr: derived(kind, 0, 2, Box::new(Expr::False), prop(1)),
            propositions: &[1],
        });
        cases.push(Case {
            name: "negated derived expression",
            expr: Box::new(Expr::Not(derived(kind, 0, 0, prop(0), prop(1)))),
            propositions: &[0, 1],
        });
    }
    cases.push(Case {
        name: "(p0 W p1) M p2",
        expr: derived(M, 1, 2, derived(W, 0, 1, prop(0), prop(1)), prop(2)),
        propositions: &[0, 1, 2],
    });
    cases.push(Case {
        name: "p2 W (p0 M p1)",
        expr: derived(W, 0, 2, prop(2), derived(M, 0, 1, prop(0), prop(1))),
        propositions: &[2, 0, 1],
    });
    cases
}

// Trace: TC-041, FR-008-AC-1
#[test]
fn lowered_graphs_rewrite_exactly_like_direct_primitive_graphs() {
    let mut statuses = Vec::new();
    for case in cases() {
        let direct = direct(&case.expr, SemanticProfile::ClosedTraceV1);
        let lowered = lowered(&case.expr, SemanticProfile::ClosedTraceV1, Mutation::None);
        assert_eq!(
            parity(&direct, &lowered, case.propositions),
            Ok(()),
            "{}",
            case.name
        );

        let report = rewrite(
            &direct,
            FORMULA_ID,
            RewriteOptions::default(),
            SOURCE_REVISION,
        );
        assert_eq!(
            report.semantic_profile, "mltl.closed-trace/v1",
            "{}",
            case.name
        );
        if let Some(output) = report.output.as_ref() {
            let conformance =
                check_equivalence(&direct, output, FORMULA_ID, ConformanceOptions::default());
            assert_eq!(
                conformance.status,
                ConformanceStatus::Equivalent,
                "{}",
                case.name
            );
        }
        statuses.push(report.status);
    }
    // The corpus exercises both a no-op and a rewriting outcome.
    assert!(statuses.contains(&RewriteStatus::Unchanged));
    assert!(statuses.contains(&RewriteStatus::Normalized));
}

// Trace: TC-042, FR-008-AC-2
#[test]
fn lowered_graphs_preserve_profile_resource_and_refusal_identities() {
    let mut exhausted = Vec::new();
    let mut unresolved = 0;
    for case in cases() {
        let direct = direct(&case.expr, SemanticProfile::ClosedTraceV1);
        let lowered = lowered(&case.expr, SemanticProfile::ClosedTraceV1, Mutation::None);
        let expected = outcomes(&direct, case.propositions);
        let observed = outcomes(&lowered, case.propositions);
        assert_eq!(observed, expected, "{}", case.name);

        for report in &observed.budgeted {
            if let Some(kind) = report.exhausted_budget {
                assert_eq!(report.status, RewriteStatus::BudgetExhausted);
                exhausted.push(kind);
            }
        }
        assert_eq!(observed.bound.binding_failure, None, "{}", case.name);
        if observed.unbound.status == RewriteStatus::UnresolvedBinding {
            unresolved += 1;
        }
        assert_eq!(
            observed
                .unbound
                .binding_failure
                .map(|failure| failure.proposition_id),
            Some(case.propositions[0]),
            "{}",
            case.name
        );

        // Online-prefix input is refused identically on both paths.
        let direct = self::direct(&case.expr, SemanticProfile::OnlinePrefixV1);
        let lowered = self::lowered(&case.expr, SemanticProfile::OnlinePrefixV1, Mutation::None);
        let expected = rewrite(
            &direct,
            FORMULA_ID,
            RewriteOptions::default(),
            SOURCE_REVISION,
        );
        let observed = rewrite(
            &lowered,
            FORMULA_ID,
            RewriteOptions::default(),
            SOURCE_REVISION,
        );
        assert_eq!(observed.status, RewriteStatus::UnsupportedProfile);
        assert_eq!(observed.semantic_profile, "mltl.online-prefix/v1");
        assert_eq!(observed, expected, "{}", case.name);
    }
    for (kind, _) in budgets() {
        assert!(exhausted.contains(&kind), "no case exhausted {kind:?}");
    }
    assert_eq!(unresolved, cases().len());
}

// Trace: TC-043, FR-008-AC-3
#[test]
fn clean_ascii_v2_parses_rewrite_like_direct_primitive_graphs() {
    use lowering::SemanticProfile as LoweringProfile;
    use tl_parse_derived::{parse_clean_ascii_v2, DerivedOperator, ParseLimits};

    assert_eq!(
        tl_parse_derived::TL_SYNTAX_REVISION,
        "8dc18eec5af227f484170362c9e8894b8531a27d"
    );
    let parsed = [
        (
            "p0 W[0,3] p1",
            derived(Kind::W, 0, 3, prop(0), prop(1)),
            vec![(DerivedOperator::WeakUntil, 2, 4)],
            &[0, 1][..],
        ),
        (
            "p0 M[2,5] p1",
            derived(Kind::M, 2, 5, prop(0), prop(1)),
            vec![(DerivedOperator::StrongRelease, 2, 4)],
            &[0, 1][..],
        ),
        (
            "true W[0,2] p1",
            derived(Kind::W, 0, 2, Box::new(Expr::True), prop(1)),
            vec![(DerivedOperator::WeakUntil, 2, 4)],
            &[1][..],
        ),
        (
            "p0 W[0,1] p1 M[1,2] p2",
            derived(
                Kind::M,
                1,
                2,
                derived(Kind::W, 0, 1, prop(0), prop(1)),
                prop(2),
            ),
            vec![
                (DerivedOperator::WeakUntil, 2, 4),
                (DerivedOperator::StrongRelease, 6, 8),
            ],
            &[0, 1, 2][..],
        ),
    ];
    for (source, expr, lowerings, propositions) in parsed {
        let report = parse_clean_ascii_v2(
            source,
            LoweringProfile::ClosedTraceV1,
            ParseLimits::default(),
        );
        assert!(
            report.diagnostics.is_empty(),
            "{source}: {:?}",
            report.diagnostics
        );
        assert_eq!(
            report
                .lowerings
                .iter()
                .map(|record| (record.kind, record.first_generated.0, record.root.0))
                .collect::<Vec<_>>(),
            lowerings,
            "{source}"
        );
        let parsed = wire(report.document.as_ref().unwrap());
        let direct = direct(&expr, SemanticProfile::ClosedTraceV1);
        // The parser attaches diagnostic spans; the semantic graph is the same.
        assert!(
            parsed.nodes().iter().all(|node| node.span.is_some()),
            "{source}"
        );
        assert_eq!(parsed.semantic_view(), direct.semantic_view(), "{source}");

        let expected = outcomes(&direct, propositions);
        let observed = outcomes(&parsed, propositions);
        for (expected, observed) in [
            (&expected.default, &observed.default),
            (&expected.bound, &observed.bound),
            (&expected.unbound, &observed.unbound),
        ]
        .into_iter()
        .chain(expected.budgeted.iter().zip(&observed.budgeted))
        {
            assert_eq!(span_free(observed), span_free(expected), "{source}");
        }
        let conformance = |outcomes: &Outcomes| {
            outcomes.conformance.as_ref().map(|report| {
                (
                    report.comparison_id.clone(),
                    report.original_sha256.clone(),
                    report.rewritten_sha256.clone(),
                    report.status,
                    report.traces_checked,
                )
            })
        };
        assert_eq!(conformance(&observed), conformance(&expected), "{source}");
    }
}

/// The span-insensitive identity of a rewrite report (FR-002-AC-4).
fn span_free(report: &RewriteReport) -> serde_json::Value {
    let mut value = serde_json::to_value(report).unwrap();
    let object = value.as_object_mut().unwrap();
    object.remove("output");
    for step in object
        .get_mut("steps")
        .and_then(serde_json::Value::as_array_mut)
        .unwrap()
    {
        assert!(step.as_object_mut().unwrap().remove("sourceSpan").is_some());
    }
    let output = report
        .output
        .as_ref()
        .map(|output| serde_json::to_value(output.semantic_view()).unwrap());
    object.insert("semantic_output".to_owned(), output.into());
    value
}

const CANONICAL_NODE_KINDS: [&str; 12] = [
    "False",
    "True",
    "Proposition",
    "Not",
    "And",
    "Or",
    "Implies",
    "Equivalent",
    "Future",
    "Globally",
    "Until",
    "Release",
];

const CANONICAL_RULE_FAMILIES: [&str; 9] = [
    "not",
    "and",
    "or",
    "implies",
    "equivalent",
    "future",
    "globally",
    "until",
    "release",
];

/// Spellings that would name a derived operator, its lowering, or a text front end.
const FORBIDDEN_TOKENS: [&str; 12] = [
    "weak",
    "strong",
    "lowering",
    "futurekind",
    "future_kind",
    "future_operators",
    "future-operators",
    "derived operator",
    "derived_operator",
    "tl_parse",
    "clean-ascii",
    "fretish",
];

/// Reports every derived-operator branch the source text could hold.
fn derived_branch_violations(path: &str, text: &str) -> Vec<String> {
    let mut violations = Vec::new();
    let lower = text.to_ascii_lowercase();
    for token in FORBIDDEN_TOKENS {
        if lower.contains(token) {
            violations.push(format!("{path}: forbidden token {token:?}"));
        }
    }
    for spelling in ["\"W\"", "'W'", "\"M\"", "'M'", "W[", "M["] {
        if text.contains(spelling) {
            violations.push(format!("{path}: derived operator spelling {spelling}"));
        }
    }
    for (index, _) in text.match_indices("NodeKind::") {
        let variant = text[index + "NodeKind::".len()..]
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect::<String>();
        if !CANONICAL_NODE_KINDS.contains(&variant.as_str()) {
            violations.push(format!("{path}: non-canonical NodeKind::{variant}"));
        }
    }
    for literal in text.split('"').skip(1).step_by(2) {
        let mut segments = literal.split('.');
        let (Some(prefix), Some(family), Some(_)) =
            (segments.next(), segments.next(), segments.next())
        else {
            continue;
        };
        if ["bool", "neg", "temporal"].contains(&prefix)
            && !CANONICAL_RULE_FAMILIES.contains(&family)
        {
            violations.push(format!("{path}: non-canonical rule family {literal:?}"));
        }
    }
    violations
}

fn crate_sources() -> Vec<(String, String)> {
    fn walk(directory: &PathBuf, files: &mut Vec<(String, String)>) {
        let mut entries = fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(&path, files);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let text = fs::read_to_string(&path).unwrap();
                files.push((path.display().to_string(), text));
            }
        }
    }

    let mut files = Vec::new();
    walk(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    files
}

// Trace: TC-044, FR-008-AC-4
#[test]
fn engine_source_has_no_derived_operator_branch() {
    let sources = crate_sources();
    assert!(
        sources.len() >= 5,
        "src/ scan found {} files",
        sources.len()
    );
    let violations = sources
        .iter()
        .flat_map(|(path, text)| derived_branch_violations(path, text))
        .collect::<Vec<_>>();
    assert_eq!(violations, Vec::<String>::new());

    // Every canonical kind is still matched somewhere, so the scan is not vacuous.
    let all = sources
        .iter()
        .map(|(_, text)| text.as_str())
        .collect::<String>();
    for kind in CANONICAL_NODE_KINDS {
        assert!(all.contains(&format!("NodeKind::{kind}")), "{kind}");
    }
    for family in CANONICAL_RULE_FAMILIES {
        assert!(all.contains(&format!(".{family}.")), "{family}");
    }

    // The lowering lane is reachable from tests only.
    let manifest =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    let production = manifest
        .split("\n[dependencies]\n")
        .nth(1)
        .and_then(|rest| rest.split("\n[").next())
        .unwrap();
    assert!(production.contains("tl-syntax = "));
    for lane in [
        "tl-parse",
        "tl-syntax-lowering",
        "8dc18eec5af227f484170362c9e8894b8531a27d",
    ] {
        assert!(
            !production.contains(lane),
            "production dependency on {lane}"
        );
    }

    // The scanner itself flags each class of derived branch.
    for synthetic in [
        "NodeKind::WeakUntil { .. } => {}",
        "match kind { \"W\" => until() }",
        "const RULE: &str = \"temporal.strong-release.lower\";",
        "fn lower_future_kind() {}",
        "use tl_parse::parse;",
        "let rule = \"neg.weakuntil.dual\";",
    ] {
        assert!(
            !derived_branch_violations("synthetic.rs", synthetic).is_empty(),
            "{synthetic}"
        );
    }
    assert!(derived_branch_violations(
        "synthetic.rs",
        "NodeKind::Until { .. } => \"temporal.until.singleton\""
    )
    .is_empty());
}

// Trace: TC-045, FR-008-AC-5
#[test]
fn wrong_lowering_and_identity_mutants_fail_parity() {
    let cases = cases();
    for mutation in ALL_MUTATIONS {
        let mut mismatched = false;
        for case in &cases {
            let direct = direct(&case.expr, SemanticProfile::ClosedTraceV1);
            let mutant = lowered(&case.expr, SemanticProfile::ClosedTraceV1, mutation);
            if mutant == direct {
                // Quantifying the right operand is the identity when both operands are one node.
                assert_eq!(
                    (mutation, case.name),
                    (Mutation::UnaryOverRight, "shared operand")
                );
                continue;
            }
            assert!(
                parity(&direct, &mutant, case.propositions).is_err(),
                "{mutation:?} survived the parity comparator on {}",
                case.name
            );
            assert!(
                outcome_parity(&direct, &mutant, case.propositions).is_err(),
                "{mutation:?} survived the engine outcomes on {}",
                case.name
            );
            if mutation == Mutation::ChangedProfile {
                let report = rewrite(
                    &mutant,
                    FORMULA_ID,
                    RewriteOptions::default(),
                    SOURCE_REVISION,
                );
                assert_eq!(report.status, RewriteStatus::UnsupportedProfile);
                continue;
            }
            let conformance =
                check_equivalence(&direct, &mutant, FORMULA_ID, ConformanceOptions::default());
            if conformance.status == ConformanceStatus::Mismatch {
                mismatched = true;
            }
            if SEMANTIC_MUTATIONS.contains(&mutation) {
                continue;
            }
            // Same semantics: only the identity and resource checks catch these.
            assert_eq!(
                conformance.status,
                ConformanceStatus::Equivalent,
                "{mutation:?}"
            );
            let direct_report = rewrite(
                &direct,
                FORMULA_ID,
                RewriteOptions::default(),
                SOURCE_REVISION,
            );
            let mutant_report = rewrite(
                &mutant,
                FORMULA_ID,
                RewriteOptions::default(),
                SOURCE_REVISION,
            );
            if SHAPE_MUTATIONS.contains(&mutation) {
                assert_ne!(mutant_report.input_sha256, direct_report.input_sha256);
            } else {
                // Diagnostic spans never enter formula identity (FR-002-AC-4), but the
                // span-bearing report still differs, so attribution drift is observable.
                assert_eq!(mutation, Mutation::SpanAttribution);
                assert_eq!(mutant_report.input_sha256, direct_report.input_sha256);
                assert_eq!(mutant_report.output_sha256, direct_report.output_sha256);
                assert_ne!(mutant_report, direct_report);
            }
            if mutation == Mutation::ExtraCharge {
                assert_ne!(mutant_report.work_units, direct_report.work_units);
            }
        }
        if SEMANTIC_MUTATIONS.contains(&mutation) {
            assert!(
                mismatched,
                "{mutation:?} never changed semantics over the corpus"
            );
        }
    }

    // The unmutated lowering passes the same comparators.
    for case in &cases {
        let direct = direct(&case.expr, SemanticProfile::ClosedTraceV1);
        let lowered = lowered(&case.expr, SemanticProfile::ClosedTraceV1, Mutation::None);
        assert_eq!(outcome_parity(&direct, &lowered, case.propositions), Ok(()));
    }
}
