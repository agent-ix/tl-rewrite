//! Direct-versus-lowered controls for the tl-syntax W/M lowering (FR-008).
//!
//! `W[a,b](p,q)` and `M[a,b](p,q)` exist only as tl-syntax lowering. The engine
//! consumes the primitive F/G/U/R and Boolean graph they lower to, so every
//! rewrite outcome of a lowered graph must equal the outcome of the same graph
//! built by hand. The lowering crates are dev-only and renamed; a lowered graph
//! reaches the production `tl_syntax` type only through the formula v1 wire.

mod common;

use std::{fs, path::Path};

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
    /// The correct lowering.
    Identity,
    /// Until and Release exchanged.
    SwapUntilRelease,
    /// Or and And exchanged.
    SwapOrAnd,
    /// Globally and Future exchanged.
    SwapGloballyFuture,
    /// The binary node's operands exchanged.
    SwapOperands,
    /// The Globally/Future node quantifies the right operand.
    UnaryOverRight,
    /// The unary node's inclusive end bound is one past the binary node's.
    WidenedEndpoint,
    /// The unary node's start bound is one later than the binary node's.
    NarrowedStart,
    /// The root joins the binary node with itself, dropping the unary side.
    DroppedUnary,
    /// The lowered document claims the online-prefix profile.
    ChangedProfile,
    /// The unary node is generated before the binary node: same semantics, other graph.
    ReorderedGenerated,
    /// An extra `And(root, root)` node: the same semantics at a different resource charge.
    ExtraCharge,
    /// Generated nodes carry a source span: the same semantic identity, other report bytes.
    SpanAttribution,
}

/// What a mutant is expected to disturb.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MutationClass {
    /// Nothing: the reference lowering.
    Identity,
    /// The formula's meaning, on at least one corpus case.
    Semantic,
    /// The semantic profile, which the engine refuses.
    Profile,
    /// The graph or resource identity, with the meaning kept.
    Shape,
    /// Only diagnostic span attribution.
    Span,
}

impl Mutation {
    /// Exhaustive, so a new variant cannot compile until it is classified.
    const fn class(self) -> MutationClass {
        match self {
            Self::Identity => MutationClass::Identity,
            Self::SwapUntilRelease
            | Self::SwapOrAnd
            | Self::SwapGloballyFuture
            | Self::SwapOperands
            | Self::UnaryOverRight
            | Self::WidenedEndpoint
            | Self::NarrowedStart
            | Self::DroppedUnary => MutationClass::Semantic,
            Self::ChangedProfile => MutationClass::Profile,
            Self::ReorderedGenerated | Self::ExtraCharge => MutationClass::Shape,
            Self::SpanAttribution => MutationClass::Span,
        }
    }

    /// Exhaustive position in [`ALL_MUTATIONS`], checked by TC-045.
    const fn index(self) -> usize {
        match self {
            Self::Identity => usize::MAX,
            Self::SwapUntilRelease => 0,
            Self::SwapOrAnd => 1,
            Self::SwapGloballyFuture => 2,
            Self::SwapOperands => 3,
            Self::UnaryOverRight => 4,
            Self::WidenedEndpoint => 5,
            Self::NarrowedStart => 6,
            Self::DroppedUnary => 7,
            Self::ChangedProfile => 8,
            Self::ReorderedGenerated => 9,
            Self::ExtraCharge => 10,
            Self::SpanAttribution => 11,
        }
    }
}

const ALL_MUTATIONS: [Mutation; 12] = [
    Mutation::SwapUntilRelease,
    Mutation::SwapOrAnd,
    Mutation::SwapGloballyFuture,
    Mutation::SwapOperands,
    Mutation::UnaryOverRight,
    Mutation::WidenedEndpoint,
    Mutation::NarrowedStart,
    Mutation::DroppedUnary,
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

fn lowering_node_id(index: usize) -> lowering::NodeId {
    lowering::NodeId(node_id(index).0)
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
        Mutation::Identity | Mutation::ChangedProfile | Mutation::ExtraCharge => {}
        Mutation::SwapOperands => match &mut binary.kind {
            K::Until { left, right, .. } | K::Release { left, right, .. } => {
                core::mem::swap(left, right);
            }
            other => panic!("unexpected lowered binary node {other:?}"),
        },
        // A singleton interval has no later start; the mutant is then the identity.
        Mutation::NarrowedStart => match &mut unary.kind {
            K::Globally { interval, .. } | K::Future { interval, .. } => {
                if interval.start() < interval.end() {
                    *interval =
                        lowering::Interval::new(interval.start() + 1, interval.end()).unwrap();
                }
            }
            other => panic!("unexpected lowered unary node {other:?}"),
        },
        Mutation::DroppedUnary => match &mut root.kind {
            K::Or { left, right } | K::And { left, right } => *right = *left,
            other => panic!("unexpected lowered root node {other:?}"),
        },
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
        lowering_node_id(nodes.len() - 1)
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
        let formula =
            lowering::Formula::new(profile, lowering_node_id(nodes.len() - 1), nodes).unwrap();
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
        assert_eq!(expansion.root(), lowering_node_id(nodes.len() - 1));
        expansion.root()
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
        root = lowering_node_id(nodes.len() - 1);
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

/// Compares engine outcomes only. It never compares the graphs themselves, so
/// a mutant must be caught by what the engine reports.
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
        let hand = direct(&case.expr, SemanticProfile::ClosedTraceV1);
        let generated = lowered(
            &case.expr,
            SemanticProfile::ClosedTraceV1,
            Mutation::Identity,
        );
        // The discriminating evidence: the lowering decodes into the production
        // 12-kind model as the same graph, spans included. Equal reports then
        // follow from deterministic rewriting and are checked as well.
        assert_eq!(generated, hand, "{}", case.name);
        assert_eq!(
            outcome_parity(&hand, &generated, case.propositions),
            Ok(()),
            "{}",
            case.name
        );

        let report = rewrite(
            &hand,
            FORMULA_ID,
            RewriteOptions::default(),
            SOURCE_REVISION,
        );
        assert_eq!(
            report.semantic_profile, "mltl.closed-trace/v1",
            "{}",
            case.name
        );
        let Some(output) = report.output.as_ref() else {
            panic!("{}: {:?} carries no output", case.name, report.status);
        };
        let conformance =
            check_equivalence(&hand, output, FORMULA_ID, ConformanceOptions::default());
        assert_eq!(
            conformance.status,
            ConformanceStatus::Equivalent,
            "{}",
            case.name
        );
        statuses.push(report.status);
    }
    // The corpus exercises both a no-op and a rewriting outcome.
    assert!(statuses.contains(&RewriteStatus::Unchanged));
    assert!(statuses.contains(&RewriteStatus::Normalized));

    // The widest representable interval. Exhaustive conformance is out of reach at
    // this horizon, so graph and rewrite-report parity are compared.
    for kind in [Kind::W, Kind::M] {
        let expr = derived(kind, u32::MAX, u32::MAX, prop(0), prop(1));
        let hand = direct(&expr, SemanticProfile::ClosedTraceV1);
        let generated = lowered(&expr, SemanticProfile::ClosedTraceV1, Mutation::Identity);
        assert_eq!(generated, hand, "{kind:?}[u32::MAX,u32::MAX]");
        assert_eq!(
            rewrite(
                &generated,
                FORMULA_ID,
                RewriteOptions::default(),
                SOURCE_REVISION
            ),
            rewrite(
                &hand,
                FORMULA_ID,
                RewriteOptions::default(),
                SOURCE_REVISION
            ),
            "{kind:?}[u32::MAX,u32::MAX]"
        );
    }
}

// Trace: TC-042, FR-008-AC-2
#[test]
fn lowered_graphs_preserve_profile_resource_and_refusal_identities() {
    let mut exhausted = Vec::new();
    let mut unresolved = 0;
    for case in cases() {
        let hand = direct(&case.expr, SemanticProfile::ClosedTraceV1);
        let generated = lowered(
            &case.expr,
            SemanticProfile::ClosedTraceV1,
            Mutation::Identity,
        );
        assert_eq!(generated, hand, "{}", case.name);
        let expected = outcomes(&hand, case.propositions);
        let observed = outcomes(&generated, case.propositions);
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
        let hand = direct(&case.expr, SemanticProfile::OnlinePrefixV1);
        let generated = lowered(
            &case.expr,
            SemanticProfile::OnlinePrefixV1,
            Mutation::Identity,
        );
        assert_eq!(generated, hand, "{}", case.name);
        let expected = rewrite(
            &hand,
            FORMULA_ID,
            RewriteOptions::default(),
            SOURCE_REVISION,
        );
        let observed = rewrite(
            &generated,
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
        (
            "p2 W[0,2] (p0 M[0,1] p1)",
            derived(
                Kind::W,
                0,
                2,
                prop(2),
                derived(Kind::M, 0, 1, prop(0), prop(1)),
            ),
            vec![
                (DerivedOperator::StrongRelease, 3, 5),
                (DerivedOperator::WeakUntil, 6, 8),
            ],
            &[2, 0, 1][..],
        ),
    ];
    let parse = |source: &str| {
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
        report
    };
    for (source, expr, lowerings, propositions) in parsed {
        let report = parse(source);
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
        let hand = direct(&expr, SemanticProfile::ClosedTraceV1);
        // The parser attaches diagnostic spans; the semantic graph is the same.
        assert!(
            parsed.nodes().iter().all(|node| node.span.is_some()),
            "{source}"
        );
        assert_eq!(parsed.semantic_view(), hand.semantic_view(), "{source}");

        let expected = outcomes(&hand, propositions);
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

    // Associativity control: the unparenthesized chain is left-associative, so the
    // right-associated graph must not match it, while its parenthesized spelling does.
    let right_associated = direct(
        &derived(
            Kind::W,
            0,
            1,
            prop(0),
            derived(Kind::M, 1, 2, prop(1), prop(2)),
        ),
        SemanticProfile::ClosedTraceV1,
    );
    let chain = wire(parse("p0 W[0,1] p1 M[1,2] p2").document.as_ref().unwrap());
    assert_ne!(chain.semantic_view(), right_associated.semantic_view());
    let grouped = wire(parse("p0 W[0,1] (p1 M[1,2] p2)").document.as_ref().unwrap());
    assert_eq!(grouped.semantic_view(), right_associated.semantic_view());
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

/// Spellings that would name a derived operator, its lowering, or a text front
/// end (Quire language, FRETish, clean-ascii). Matched case-insensitively.
/// A bare "quire" is not listed because it occurs inside "requirement".
const FORBIDDEN_TOKENS: [&str; 22] = [
    "weak",
    "strong",
    "unless",
    "desugar",
    "lowering",
    "futurekind",
    "future_kind",
    "future_operators",
    "future-operators",
    "derived operator",
    "derived_operator",
    "derived-operator",
    "derivedoperator",
    "tl_parse",
    "tl-parse",
    "clean-ascii",
    "clean_ascii",
    "fretish",
    "quire_language",
    "quire-language",
    "quire language",
    "quire::",
];

/// The rule-id prefixes the catalog uses for primitive families.
const RULE_PREFIXES: [&str; 3] = ["bool", "neg", "temporal"];

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
    // An alias would hide variants from the `NodeKind::` check below.
    if text.contains("NodeKind as") {
        violations.push(format!("{path}: aliased NodeKind"));
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
    // Each rule-shaped literal is read from its own opening quote, so a stray
    // quote elsewhere cannot shift which spans are treated as literals.
    for prefix in RULE_PREFIXES {
        let opening = format!("\"{prefix}.");
        for (index, _) in text.match_indices(&opening) {
            let literal = text[index + 1..].split('"').next().unwrap_or_default();
            let family = literal.split('.').nth(1).unwrap_or_default();
            if !CANONICAL_RULE_FAMILIES.contains(&family) {
                violations.push(format!("{path}: non-canonical rule family {literal:?}"));
            }
        }
    }
    violations
}

/// Every `.rs` file under `src/` and `examples/`, the engine and the producers
/// that feed the gates.
fn crate_sources() -> Vec<(String, String)> {
    fn walk(directory: &Path, files: &mut Vec<(String, String)>) {
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
    for directory in ["src", "examples"] {
        walk(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join(directory),
            &mut files,
        );
    }
    files
}

/// The manifest's non-dev dependency sections, each as its header and body.
fn production_dependency_sections(manifest: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let mut current: Option<String> = None;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            sections.extend(current.take());
            let header = trimmed.to_ascii_lowercase();
            let production = (header.contains("dependencies") || header.contains("dependency"))
                && !header.contains("dev-dependencies")
                && !header.contains("dev_dependencies");
            if production {
                current = Some(format!("{trimmed}\n"));
            }
        } else if let Some(section) = current.as_mut() {
            section.push_str(line);
            section.push('\n');
        }
    }
    sections.extend(current);
    sections
}

// Trace: TC-044, FR-008-AC-4
#[test]
fn engine_source_has_no_derived_operator_branch() {
    let sources = crate_sources();
    for directory in ["/src/", "/examples/"] {
        assert!(
            sources
                .iter()
                .filter(|(path, _)| path.contains(directory))
                .count()
                >= 3,
            "the {directory} scan is nearly empty"
        );
    }
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

    // The executable catalog names only primitive families. `west.*` rules are
    // the retained, excluded nested Until/Release rewrites.
    for rule in tl_rewrite::catalog().rules {
        let mut segments = rule.id.split('.');
        let (prefix, family) = (segments.next().unwrap(), segments.next().unwrap());
        match prefix {
            "west" => assert!(
                ["nested-until-right", "nested-release-right"].contains(&family),
                "{}",
                rule.id
            ),
            _ => {
                assert!(RULE_PREFIXES.contains(&prefix), "{}", rule.id);
                assert!(CANONICAL_RULE_FAMILIES.contains(&family), "{}", rule.id);
            }
        }
    }

    // The lowering lane is reachable from tests only: no non-dev dependency
    // section, in any table form, names it, and the production tl-syntax pin is
    // the revision the crate publishes.
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    let production = production_dependency_sections(&manifest);
    let joined = production.concat();
    assert!(
        joined.contains(&format!("rev = \"{}\"", tl_rewrite::TL_SYNTAX_REVISION)),
        "{joined}"
    );
    for lane in [
        "tl-parse",
        "tl-syntax-lowering",
        "8dc18eec5af227f484170362c9e8894b8531a27d",
        "9ca856b4c040fc2c3329b6defd26a1c9b57de748",
    ] {
        assert!(!joined.contains(lane), "production dependency on {lane}");
    }
    let synthetic_manifest = "[package]\nname = \"x\"\n\n[dependencies]\ntl-syntax = \"1\"\n\n\
        [dev-dependencies]\ntl-parse = \"1\"\n\n[dependencies.tl-syntax-lowering]\nrev = \"a\"\n\n\
        [target.'cfg(unix)'.dependencies]\nfoo = \"1\"\n\n[build-dependencies]\nbar = \"1\"\n";
    let sections = production_dependency_sections(synthetic_manifest);
    assert_eq!(sections.len(), 4, "{sections:?}");
    assert!(!sections.concat().contains("tl-parse"));
    assert!(sections.concat().contains("tl-syntax-lowering"));

    // The scanner itself flags each class of derived branch. Each synthetic input
    // is caught by exactly the named check, so no check is covered only by another.
    for (synthetic, expected) in [
        (
            "NodeKind::Next { .. } => {}",
            "synthetic.rs: non-canonical NodeKind::Next",
        ),
        (
            "use tl_syntax::NodeKind as K;",
            "synthetic.rs: aliased NodeKind",
        ),
        (
            "match kind { \"W\" => until() }",
            "synthetic.rs: derived operator spelling \"W\"",
        ),
        (
            "let quote = '\"'; const RULE: &str = \"temporal.next.dual\";",
            "synthetic.rs: non-canonical rule family \"temporal.next.dual\"",
        ),
        (
            "let rule = \"neg.xor.dual\";",
            "synthetic.rs: non-canonical rule family \"neg.xor.dual\"",
        ),
        (
            "fn lower_future_kind() {}",
            "synthetic.rs: forbidden token \"future_kind\"",
        ),
        (
            "use tl_parse::parse;",
            "synthetic.rs: forbidden token \"tl_parse\"",
        ),
        (
            "enum DerivedOperator {}",
            "synthetic.rs: forbidden token \"derivedoperator\"",
        ),
        (
            "fn read_quire_language() {}",
            "synthetic.rs: forbidden token \"quire_language\"",
        ),
        (
            "mod fretish_front_end {}",
            "synthetic.rs: forbidden token \"fretish\"",
        ),
        (
            "fn desugar_unless() {}",
            "synthetic.rs: forbidden token \"unless\"",
        ),
    ] {
        let violations = derived_branch_violations("synthetic.rs", synthetic);
        assert!(
            violations.iter().any(|violation| violation == expected),
            "{synthetic}: {violations:?}"
        );
    }
    assert_eq!(
        derived_branch_violations(
            "synthetic.rs",
            "NodeKind::Until { .. } => \"temporal.until.singleton\" // requirement"
        ),
        Vec::<String>::new()
    );
}

// Trace: TC-045, FR-008-AC-5
#[test]
fn wrong_lowering_and_identity_mutants_fail_parity() {
    // The mutant list is the enum, in order, with nothing left out.
    for (position, mutation) in ALL_MUTATIONS.into_iter().enumerate() {
        assert_eq!(mutation.index(), position, "{mutation:?}");
        assert_ne!(mutation.class(), MutationClass::Identity, "{mutation:?}");
    }
    assert_eq!(Mutation::Identity.class(), MutationClass::Identity);

    let cases = cases();
    for mutation in ALL_MUTATIONS {
        let class = mutation.class();
        let mut changed = false;
        let mut mismatched = false;
        for case in &cases {
            let hand = direct(&case.expr, SemanticProfile::ClosedTraceV1);
            let mutant = lowered(&case.expr, SemanticProfile::ClosedTraceV1, mutation);
            if mutant == hand {
                // Where both operands are one node, retargeting or exchanging them is
                // the identity; a singleton interval has no later start.
                let expected_identity = match mutation {
                    Mutation::UnaryOverRight | Mutation::SwapOperands => {
                        case.name == "shared operand"
                    }
                    Mutation::NarrowedStart => {
                        ["distinct propositions", "negated derived expression"].contains(&case.name)
                    }
                    _ => false,
                };
                assert!(
                    expected_identity,
                    "{mutation:?} is the identity on {}",
                    case.name
                );
                continue;
            }
            changed = true;
            // The comparator sees engine outcomes only, never the graphs.
            assert!(
                outcome_parity(&hand, &mutant, case.propositions).is_err(),
                "{mutation:?} survived the engine outcomes on {}",
                case.name
            );
            if class == MutationClass::Profile {
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
                check_equivalence(&hand, &mutant, FORMULA_ID, ConformanceOptions::default());
            if conformance.status == ConformanceStatus::Mismatch {
                mismatched = true;
            }
            if class == MutationClass::Semantic {
                continue;
            }
            // Same semantics: only the identity and resource checks catch these.
            assert_eq!(
                conformance.status,
                ConformanceStatus::Equivalent,
                "{mutation:?}"
            );
            let hand_report = rewrite(
                &hand,
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
            match class {
                MutationClass::Shape => {
                    assert_ne!(mutant_report.input_sha256, hand_report.input_sha256);
                }
                MutationClass::Span => {
                    // Diagnostic spans never enter formula identity (FR-002-AC-4), but
                    // the span-bearing report still differs, so attribution drift is
                    // observable.
                    assert_eq!(mutant_report.input_sha256, hand_report.input_sha256);
                    assert_eq!(mutant_report.output_sha256, hand_report.output_sha256);
                    assert_ne!(mutant_report, hand_report);
                }
                MutationClass::Identity | MutationClass::Semantic | MutationClass::Profile => {
                    unreachable!("{mutation:?}")
                }
            }
            if mutation == Mutation::ExtraCharge {
                assert_ne!(mutant_report.work_units, hand_report.work_units);
            }
        }
        assert!(changed, "{mutation:?} never changed a corpus graph");
        if class == MutationClass::Semantic {
            assert!(
                mismatched,
                "{mutation:?} never changed semantics over the corpus"
            );
        }
    }

    // The reference lowering passes the same comparator.
    for case in &cases {
        let hand = direct(&case.expr, SemanticProfile::ClosedTraceV1);
        let generated = lowered(
            &case.expr,
            SemanticProfile::ClosedTraceV1,
            Mutation::Identity,
        );
        assert_eq!(outcome_parity(&hand, &generated, case.propositions), Ok(()));
    }
}
