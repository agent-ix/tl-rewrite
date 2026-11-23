//! Solver-backed equivalence (FR-091), compiled only with `--features solver`.
//!
//! UNBUILT: written without any cargo run (TL-DECISIONS-2026-11-23); every
//! test here is a deferred verification in the PLAN's table.
#![cfg(feature = "solver")]

mod common;

use std::{fs, path::Path, time::Duration};

use common::{document, proposition};
use proptest::prelude::*;
use serde::Deserialize;
use tl_mltl::{evaluate_closed, EvaluationLimits, TruthValue};
use tl_rewrite::{
    check_equivalence, check_equivalence_with_solver, rewrite, solver_conformance_from_result,
    ConformanceOptions, ConformanceReport, ConformanceStatus, RewriteOptions,
    SolverConformanceOptions, SolverConformanceReason, SolverConformanceReport,
    SOLVER_CONFORMANCE_SCHEMA,
};
use tl_solve::{
    check_satisfiable, Capabilities, Cnf, Disposition, EncodeLimits, FrameRequest,
    InProcessBackend, Lit, NonAnswer, RecheckLimits, SatBackend, SatRequest, SolveLimits,
    SolveOutcome, SolverIdentity,
};
use tl_syntax::{
    FormulaDocument, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile,
};

fn solve(
    original: &FormulaDocument,
    rewritten: &FormulaDocument,
    options: SolverConformanceOptions,
) -> SolverConformanceReport {
    let mut backend = InProcessBackend;
    check_equivalence_with_solver(original, rewritten, "solver", options, &mut backend)
}

fn enumerate(original: &FormulaDocument, rewritten: &FormulaDocument) -> ConformanceReport {
    check_equivalence(original, rewritten, "enumeration", ConformanceOptions::default())
}

/// Independent of the production re-check: evaluates `formula` on `trace`
/// with tl-mltl's default limits.
fn independent_verdict(formula: &FormulaDocument, trace: &[Vec<u32>]) -> TruthValue {
    let instants: Vec<Vec<PropositionId>> = trace
        .iter()
        .map(|instant| instant.iter().copied().map(PropositionId).collect())
        .collect();
    evaluate_closed(
        formula.validate().unwrap(),
        "independent",
        &instants,
        "independent-trace".to_owned(),
        EvaluationLimits::default(),
    )
    .unwrap()
    .verdict
}

fn assert_confirmed_mismatch(
    original: &FormulaDocument,
    rewritten: &FormulaDocument,
    report: &SolverConformanceReport,
) {
    assert_eq!(report.status, ConformanceStatus::Mismatch);
    let trace = report.counterexample.as_ref().expect("a mismatch carries a trace");
    assert_eq!(
        u64::try_from(trace.len()).unwrap(),
        report.trace_length.unwrap()
    );
    let left = independent_verdict(original, trace);
    let right = independent_verdict(rewritten, trace);
    assert_ne!(left, right, "the counterexample must separate the pair");
    assert_eq!(report.original_verdict, Some(left));
    assert_eq!(report.rewritten_verdict, Some(right));
}

/// The agreement oracle of FR-091-AC-2: wherever enumeration decides, the
/// solver path gives the same status, and every solver mismatch is confirmed.
fn assert_agrees(original: &FormulaDocument, rewritten: &FormulaDocument) {
    let enumerated = enumerate(original, rewritten);
    if enumerated.status == ConformanceStatus::NonConclusive {
        return;
    }
    let solved = solve(original, rewritten, SolverConformanceOptions::default());
    assert_eq!(solved.schema_version, SOLVER_CONFORMANCE_SCHEMA);
    assert_eq!(
        solved.status, enumerated.status,
        "solver {solved:?} disagrees with enumeration {enumerated:?}"
    );
    assert_eq!(solved.trace_length, enumerated.trace_length);
    match solved.status {
        ConformanceStatus::Equivalent => {
            assert!(solved.basis.is_some());
            assert!(solved.solver.is_some());
            assert!(solved.counterexample.is_none());
        }
        ConformanceStatus::Mismatch => assert_confirmed_mismatch(original, rewritten, &solved),
        ConformanceStatus::NonConclusive => unreachable!("compared above"),
    }
}

fn read_manifest(path: &str) -> serde_json::Value {
    let bytes = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect(path);
    serde_json::from_slice(&bytes).expect(path)
}

#[derive(Deserialize)]
struct RuleCase {
    id: String,
    disposition: String,
    document: FormulaDocument,
}

// Trace: TC-976, FR-091-AC-2
#[test]
fn solver_agrees_with_enumeration_on_every_enabled_closed_trace_rule_instance() {
    let manifest = read_manifest("corpus/rules/manifest.json");
    let cases: Vec<RuleCase> = serde_json::from_value(manifest["cases"].clone()).unwrap();
    let mut compared = 0_usize;
    for case in cases {
        if case.disposition != "enabled"
            || case.document.semantic_profile() != SemanticProfile::ClosedTraceV1
        {
            continue;
        }
        let report = rewrite(&case.document, &case.id, RewriteOptions::default(), "solver");
        let Some(output) = report.output.as_ref() else {
            continue;
        };
        assert_agrees(&case.document, output);
        // Both directions, and the unrewritten identity pair.
        assert_agrees(output, &case.document);
        assert_agrees(&case.document, &case.document);
        compared += 1;
    }
    assert!(compared > 0, "the rule corpus holds closed-trace instances");
}

// Trace: TC-976, TC-977, FR-091-AC-2, FR-091-AC-3
#[test]
fn solver_agrees_with_enumeration_on_the_retained_mismatch_corpus() {
    let manifest = read_manifest("corpus/counterexamples/manifest.json");
    let mut compared = 0_usize;
    for case in manifest["cases"].as_array().unwrap() {
        if case["kind"] != "mismatch" {
            continue;
        }
        let original: FormulaDocument = serde_json::from_value(case["original"].clone()).unwrap();
        let rewritten: FormulaDocument =
            serde_json::from_value(case["rewritten"].clone()).unwrap();
        assert_eq!(enumerate(&original, &rewritten).status, ConformanceStatus::Mismatch);
        assert_agrees(&original, &rewritten);
        compared += 1;
    }
    assert_eq!(compared, 5, "the corpus declares five mismatch cases");
}

/// A small closed-trace formula over at most three propositions with
/// intervals inside [0, 2] (the enumeration-decidable small scope).
fn small_formula() -> impl Strategy<Value = FormulaDocument> {
    let leaf = prop_oneof![
        Just(vec![Node::new(NodeKind::True)]),
        Just(vec![Node::new(NodeKind::False)]),
        (0_u32..3).prop_map(|id| vec![proposition(id)]),
    ];
    let tree = leaf.prop_recursive(3, 12, 2, |inner| {
        let interval = (0_u32..=2, 0_u32..=2).prop_map(|(a, b)| {
            Interval::new(a.min(b), a.max(b)).unwrap()
        });
        prop_oneof![
            inner.clone().prop_map(|nodes| unary(nodes, |operand| NodeKind::Not { operand })),
            (inner.clone(), interval.clone()).prop_map(|(nodes, interval)| {
                unary(nodes, |operand| NodeKind::Future { interval, operand })
            }),
            (inner.clone(), interval.clone()).prop_map(|(nodes, interval)| {
                unary(nodes, |operand| NodeKind::Globally { interval, operand })
            }),
            (inner.clone(), inner.clone()).prop_map(|(left, right)| {
                binary(left, right, |left, right| NodeKind::And { left, right })
            }),
            (inner.clone(), inner.clone()).prop_map(|(left, right)| {
                binary(left, right, |left, right| NodeKind::Or { left, right })
            }),
            (inner.clone(), inner.clone(), interval.clone()).prop_map(|(left, right, interval)| {
                binary(left, right, |left, right| NodeKind::Until {
                    interval,
                    left,
                    right,
                })
            }),
            (inner.clone(), inner, interval).prop_map(|(left, right, interval)| {
                binary(left, right, |left, right| NodeKind::Release {
                    interval,
                    left,
                    right,
                })
            }),
        ]
    });
    tree.prop_map(|nodes| document(SemanticProfile::ClosedTraceV1, nodes))
}

fn last(nodes: &[Node]) -> NodeId {
    NodeId(u32::try_from(nodes.len() - 1).unwrap())
}

fn unary(mut nodes: Vec<Node>, kind: impl Fn(NodeId) -> NodeKind) -> Vec<Node> {
    let operand = last(&nodes);
    nodes.push(Node::new(kind(operand)));
    nodes
}

/// Concatenates two graphs (the right one shifted) under a binary root.
fn binary(
    mut left: Vec<Node>,
    right: Vec<Node>,
    kind: impl Fn(NodeId, NodeId) -> NodeKind,
) -> Vec<Node> {
    let left_root = last(&left);
    let offset = u32::try_from(left.len()).unwrap();
    for node in right {
        left.push(Node::new(shift(node.kind, offset)));
    }
    let right_root = last(&left);
    left.push(Node::new(kind(left_root, right_root)));
    left
}

fn shift(kind: NodeKind, offset: u32) -> NodeKind {
    let s = |id: NodeId| NodeId(id.0 + offset);
    match kind {
        NodeKind::Not { operand } => NodeKind::Not { operand: s(operand) },
        NodeKind::Future { interval, operand } => NodeKind::Future {
            interval,
            operand: s(operand),
        },
        NodeKind::Globally { interval, operand } => NodeKind::Globally {
            interval,
            operand: s(operand),
        },
        NodeKind::And { left, right } => NodeKind::And {
            left: s(left),
            right: s(right),
        },
        NodeKind::Or { left, right } => NodeKind::Or {
            left: s(left),
            right: s(right),
        },
        NodeKind::Until {
            interval,
            left,
            right,
        } => NodeKind::Until {
            interval,
            left: s(left),
            right: s(right),
        },
        NodeKind::Release {
            interval,
            left,
            right,
        } => NodeKind::Release {
            interval,
            left: s(left),
            right: s(right),
        },
        NodeKind::False | NodeKind::True | NodeKind::Proposition { .. } => kind,
        NodeKind::Implies { .. }
        | NodeKind::Equivalent { .. }
        | NodeKind::Once { .. }
        | NodeKind::Historically { .. }
        | NodeKind::StrongPrevious { .. }
        | NodeKind::Since { .. }
        | NodeKind::Triggered { .. }
        | NodeKind::LtlfFuture { .. }
        | NodeKind::LtlfGlobally { .. }
        | NodeKind::LtlfUntil { .. }
        | NodeKind::LtlfRelease { .. }
        | NodeKind::StrongNext { .. }
        | NodeKind::WeakNext { .. } => panic!("the generator never emits {kind:?}"),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // Trace: TC-976, FR-091-AC-2
    #[test]
    fn solver_agrees_with_enumeration_on_random_small_pairs(
        original in small_formula(),
        rewritten in small_formula(),
    ) {
        assert_agrees(&original, &rewritten);
        // The engine's own output is the population FR-091 cares about most.
        let report = rewrite(&original, "property", RewriteOptions::default(), "solver");
        if let Some(output) = report.output.as_ref() {
            assert_agrees(&original, output);
        }
    }
}

fn future(interval: Interval) -> FormulaDocument {
    document(
        SemanticProfile::ClosedTraceV1,
        vec![
            proposition(0),
            Node::new(NodeKind::Future {
                interval,
                operand: NodeId(0),
            }),
        ],
    )
}

fn globally(interval: Interval) -> FormulaDocument {
    document(
        SemanticProfile::ClosedTraceV1,
        vec![
            proposition(0),
            Node::new(NodeKind::Globally {
                interval,
                operand: NodeId(0),
            }),
        ],
    )
}

// Trace: TC-977, FR-091-AC-3
#[test]
fn a_returned_trace_on_which_the_verdicts_agree_is_a_typed_failure_not_a_mismatch() {
    // F[0,1] p and G[0,1] p differ, but every trace satisfying G[0,1] p also
    // satisfies F[0,1] p. A tl-solve result whose witness was solved for
    // G[0,1] p alone (the seeded fault: a "counterexample" that does not
    // separate the pair) must be refused by tl-rewrite's own re-check.
    let interval = Interval::new(0, 1).unwrap();
    let original = future(interval);
    let rewritten = globally(interval);
    let faulty = check_satisfiable(
        &SatRequest {
            formula: rewritten.validate().unwrap(),
            formula_id: "seeded-fault",
            frame: FrameRequest::Closed {
                trace_length: Some(2),
            },
            encode_limits: EncodeLimits::default(),
            solve_limits: SolveLimits::default(),
            recheck: RecheckLimits::default(),
        },
        &mut InProcessBackend,
    );
    assert_eq!(faulty.disposition, Disposition::Satisfiable);
    let report = solver_conformance_from_result(
        &original,
        &rewritten,
        "seeded-fault",
        SolverConformanceOptions::default(),
        &faulty,
    );
    assert_eq!(report.status, ConformanceStatus::NonConclusive);
    assert_eq!(
        report.reason,
        Some(SolverConformanceReason::CounterexampleNotConfirmed)
    );
    assert!(report.counterexample.is_none());
    let trace = report.diagnostic_trace.as_ref().unwrap();
    assert_eq!(
        independent_verdict(&original, trace),
        independent_verdict(&rewritten, trace)
    );

    // The genuine run on the same pair is a confirmed mismatch.
    let genuine = solve(&original, &rewritten, SolverConformanceOptions::default());
    assert_confirmed_mismatch(&original, &rewritten, &genuine);
}

/// p0 ∧ (p1 ∧ (p2 ∧ (p3 ∧ p4))) with the root operands in either order.
fn five_proposition_conjunction(swapped: bool) -> FormulaDocument {
    let mut nodes: Vec<Node> = (0..5).map(proposition).collect();
    let and = |left: u32, right: u32| {
        Node::new(NodeKind::And {
            left: NodeId(left),
            right: NodeId(right),
        })
    };
    nodes.push(and(3, 4)); // 5
    nodes.push(and(2, 5)); // 6
    nodes.push(and(1, 6)); // 7
    nodes.push(if swapped { and(7, 0) } else { and(0, 7) }); // 8
    document(SemanticProfile::ClosedTraceV1, nodes)
}

/// ¬F[0,b] ¬p, the dual spelling of G[0,b] p.
fn dual_globally(interval: Interval) -> FormulaDocument {
    document(
        SemanticProfile::ClosedTraceV1,
        vec![
            proposition(0),
            Node::new(NodeKind::Not { operand: NodeId(0) }),
            Node::new(NodeKind::Future {
                interval,
                operand: NodeId(1),
            }),
            Node::new(NodeKind::Not { operand: NodeId(2) }),
        ],
    )
}

fn large_recheck() -> SolverConformanceOptions {
    SolverConformanceOptions {
        // FR-091: intervals near 10,000 may need a raised re-check budget.
        recheck_limits: RecheckLimits {
            max_node_evaluations: 200_000_000,
        },
        ..SolverConformanceOptions::default()
    }
}

// Trace: TC-978, FR-091-AC-4
#[test]
fn pairs_beyond_the_enumeration_ceilings_are_decided_by_the_solver() {
    // Proposition ceiling: five propositions against enumeration's four.
    let left = five_proposition_conjunction(false);
    let right = five_proposition_conjunction(true);
    let enumerated = enumerate(&left, &right);
    assert_eq!(enumerated.status, ConformanceStatus::NonConclusive);
    let solved = solve(&left, &right, SolverConformanceOptions::default());
    assert_eq!(solved.status, ConformanceStatus::Equivalent);

    // Horizon and trace-domain ceilings, at interval 10,000.
    let wide = Interval::new(0, 10_000).unwrap();
    let enumerated = enumerate(&globally(wide), &dual_globally(wide));
    assert_eq!(enumerated.status, ConformanceStatus::NonConclusive);
    let solved = solve(&globally(wide), &dual_globally(wide), large_recheck());
    assert_eq!(solved.status, ConformanceStatus::Equivalent);
    assert_eq!(solved.trace_length, Some(10_001));

    let shifted = Interval::new(1, 10_000).unwrap();
    let solved = solve(&future(wide), &future(shifted), large_recheck());
    assert_confirmed_mismatch(&future(wide), &future(shifted), &solved);
}

/// A backend that declares `capabilities` and always answers `outcome`.
struct Scripted {
    capabilities: Capabilities,
    outcome: SolveOutcome,
}

impl SatBackend for Scripted {
    fn identity(&self) -> SolverIdentity {
        SolverIdentity {
            name: "scripted".to_owned(),
            version: "0".to_owned(),
            command_line: None,
        }
    }

    fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    fn solve(&mut self, _cnf: &Cnf, _assumptions: &[Lit], _limits: &SolveLimits) -> SolveOutcome {
        self.outcome.clone()
    }
}

// Trace: TC-978, FR-091-AC-4
#[test]
fn every_limit_and_refusal_is_non_conclusive_with_a_distinct_reason() {
    let interval = Interval::new(0, 2).unwrap();
    let original = future(interval);
    let rewritten = globally(interval);
    let run = |backend: &mut dyn SatBackend, options: SolverConformanceOptions| {
        check_equivalence_with_solver(&original, &rewritten, "limits", options, backend)
    };

    let conflict = run(
        &mut Scripted {
            capabilities: Capabilities {
                conflict_limit: true,
                ..Capabilities::default()
            },
            outcome: SolveOutcome::NonAnswer(NonAnswer::ConflictLimit),
        },
        SolverConformanceOptions {
            solve_limits: SolveLimits {
                max_conflicts: Some(1),
                ..SolveLimits::default()
            },
            ..SolverConformanceOptions::default()
        },
    );
    let time = run(
        &mut Scripted {
            capabilities: Capabilities {
                time_limit: true,
                ..Capabilities::default()
            },
            outcome: SolveOutcome::NonAnswer(NonAnswer::TimeLimit),
        },
        SolverConformanceOptions {
            solve_limits: SolveLimits {
                wall_time: Some(Duration::from_millis(1)),
                ..SolveLimits::default()
            },
            ..SolverConformanceOptions::default()
        },
    );
    // Refused backend: a conflict limit the backend does not declare.
    let refused = run(
        &mut Scripted {
            capabilities: Capabilities::default(),
            outcome: SolveOutcome::NonAnswer(NonAnswer::ConflictLimit),
        },
        SolverConformanceOptions {
            solve_limits: SolveLimits {
                max_conflicts: Some(1),
                ..SolveLimits::default()
            },
            ..SolverConformanceOptions::default()
        },
    );
    // Re-check budget stop: a real solve of a non-equivalent pair with a
    // one-evaluation re-check budget.
    let recheck = run(
        &mut InProcessBackend,
        SolverConformanceOptions {
            recheck_limits: RecheckLimits {
                max_node_evaluations: 1,
            },
            ..SolverConformanceOptions::default()
        },
    );

    let reasons = [
        (conflict, SolverConformanceReason::ConflictLimit),
        (time, SolverConformanceReason::TimeLimit),
        (refused, SolverConformanceReason::CapabilityUnsupported),
        (recheck, SolverConformanceReason::RecheckLimit),
    ];
    for (report, reason) in &reasons {
        assert_eq!(report.status, ConformanceStatus::NonConclusive, "{report:?}");
        assert_eq!(report.reason, Some(*reason), "{report:?}");
        assert!(report.counterexample.is_none());
        assert!(report.basis.is_none());
    }
    assert_eq!(reasons[3].0.recheck_node_evaluations, 1);
}

// Trace: TC-975, FR-091-AC-1
#[test]
fn the_solver_report_round_trips_under_its_own_schema() {
    let interval = Interval::new(0, 2).unwrap();
    let report = solve(
        &future(interval),
        &globally(interval),
        SolverConformanceOptions::default(),
    );
    let value = serde_json::to_value(&report).unwrap();
    assert_eq!(value["schemaVersion"], SOLVER_CONFORMANCE_SCHEMA);
    let decoded: SolverConformanceReport = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(decoded, report);
    let mut unknown = value;
    unknown["extra"] = serde_json::Value::Bool(true);
    assert!(serde_json::from_value::<SolverConformanceReport>(unknown).is_err());
}
