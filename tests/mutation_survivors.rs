//! TL-371: behavior tests for mutants that survived the first full cargo-mutants
//! run of the rewrite engine. Each test names the behavior it pins down.

mod common;

use common::{document, proposition};
use tl_rewrite::report::read_with_context;
use tl_rewrite::{
    check_equivalence, infinite_catalog, replay_infinite, rewrite, rewrite_infinite,
    rewrite_with_context, BudgetKind, ConformanceOptions, ConformanceStatus,
    InfiniteRewriteFailure, RecordLimits, RewriteBudgets, RewriteOptions, RewriteReport,
    RewriteStatus, MAX_INFINITE_REPORT_BYTES,
};
use tl_syntax::{
    InfiniteClock, InfiniteFormulaDocument, InfiniteNode, InfiniteNodeKind as K, Interval, Node,
    NodeId, NodeKind, OwnedSignalDeclaration, PropositionBinding, PropositionId, SemanticProfile,
    SignalCatalogDocument, SignalDomain, SignalId,
};

fn options_with_nodes(max_nodes: u32) -> RewriteOptions {
    RewriteOptions {
        budgets: RewriteBudgets {
            max_nodes,
            ..RewriteBudgets::default()
        },
        ..RewriteOptions::default()
    }
}

/// Runs `work` on its own thread and fails the test if it has not finished
/// within the deadline, so a mutated traversal that never ends fails the test
/// instead of hanging the suite.
fn within_deadline<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(work());
    });
    receiver
        .recv_timeout(std::time::Duration::from_secs(30))
        .expect("the rewrite did not terminate")
}

type BinaryTemporal = fn(Interval, NodeId, NodeId) -> NodeKind;

// Trace: TC-005, FR-002-AC-1
#[test]
fn node_budget_admits_a_graph_exactly_at_the_limit_and_refuses_one_past() {
    // `p -> q` has three nodes and rewrites to a four-node `!p | q`.
    let implication = document(
        SemanticProfile::ClosedTraceV1,
        vec![
            proposition(0),
            proposition(1),
            Node::new(NodeKind::Implies {
                left: NodeId(0),
                right: NodeId(1),
            }),
        ],
    );
    let at_output = rewrite(&implication, "grows", options_with_nodes(4), "source");
    assert_eq!(at_output.status, RewriteStatus::Normalized);
    assert_eq!(at_output.output.as_ref().map(|d| d.nodes().len()), Some(4));
    // The three-node input fits, the four-node rewritten graph does not.
    let past_output = rewrite(&implication, "grows", options_with_nodes(3), "source");
    assert_eq!(past_output.status, RewriteStatus::BudgetExhausted);
    assert_eq!(past_output.exhausted_budget, Some(BudgetKind::Nodes));

    // `true & p` has three nodes and rewrites to one.
    let reducible = document(
        SemanticProfile::ClosedTraceV1,
        vec![
            Node::new(NodeKind::True),
            proposition(3),
            Node::new(NodeKind::And {
                left: NodeId(0),
                right: NodeId(1),
            }),
        ],
    );
    let at_input = rewrite(&reducible, "shrinks", options_with_nodes(3), "source");
    assert_eq!(at_input.status, RewriteStatus::Normalized);
    let past_input = rewrite(&reducible, "shrinks", options_with_nodes(2), "source");
    assert_eq!(past_input.status, RewriteStatus::BudgetExhausted);
    assert_eq!(past_input.exhausted_budget, Some(BudgetKind::Nodes));
}

// Trace: TC-005, FR-002-AC-1
#[test]
fn compaction_visits_a_shared_subgraph_once() {
    // Each node is both operands of the next, so a traversal that revisits a
    // shared node does 2^40 visits.
    let mut nodes = vec![proposition(0)];
    for index in 0..40_u32 {
        nodes.push(Node::new(NodeKind::And {
            left: NodeId(index),
            right: NodeId(index),
        }));
    }
    let shared = document(SemanticProfile::ClosedTraceV1, nodes);
    let report =
        within_deadline(move || rewrite(&shared, "shared", RewriteOptions::default(), "source"));
    assert_eq!(report.status, RewriteStatus::Normalized);
    assert_eq!(report.output.as_ref().map(|d| d.nodes().len()), Some(1));
}

// Trace: TC-005, FR-002-AC-1
#[test]
fn a_singleton_until_or_release_reports_no_step_inside_the_dropped_operand() {
    // The singleton rule keeps only the right operand, so a rewrite inside the
    // discarded left operand is not part of the derivation.
    let interval = Interval::new(0, 0).unwrap();
    let kinds: [(&str, BinaryTemporal); 2] = [
        ("temporal.until.singleton", |interval, left, right| {
            NodeKind::Until {
                interval,
                left,
                right,
            }
        }),
        ("temporal.release.singleton", |interval, left, right| {
            NodeKind::Release {
                interval,
                left,
                right,
            }
        }),
    ];
    for (rule, build) in kinds {
        let input = document(
            SemanticProfile::ClosedTraceV1,
            vec![
                proposition(0),
                Node::new(NodeKind::True),
                Node::new(NodeKind::And {
                    left: NodeId(0),
                    right: NodeId(1),
                }),
                proposition(1),
                Node::new(build(interval, NodeId(2), NodeId(3))),
            ],
        );
        let report = rewrite(&input, "singleton", RewriteOptions::default(), "source");
        assert_eq!(report.status, RewriteStatus::Normalized, "{rule}");
        let rules: Vec<&str> = report.steps.iter().map(|s| s.rule_id.as_str()).collect();
        assert_eq!(rules, vec![rule]);
    }
}

// Trace: TC-005, FR-010-AC-1
#[test]
fn only_the_exact_once_one_one_interval_folds_to_strong_previous() {
    let past = |start: u32, end: u32| {
        tl_syntax::FormulaDocument::new_v2(
            SemanticProfile::OriginCompleteHistoryV1,
            NodeId(1),
            vec![
                proposition(0),
                Node::new(NodeKind::Once {
                    interval: Interval::new(start, end).unwrap(),
                    operand: NodeId(0),
                }),
            ],
        )
        .unwrap()
    };
    let steps = |start, end| {
        rewrite(
            &past(start, end),
            "once",
            RewriteOptions::default(),
            "source",
        )
        .steps
        .into_iter()
        .map(|step| step.rule_id)
        .collect::<Vec<_>>()
    };
    assert_eq!(steps(1, 1), vec!["past.once.strong-previous"]);
    // A start of one alone, or an end of one alone, is a different interval.
    assert!(steps(1, 3).is_empty());
    assert!(steps(0, 1).is_empty());
    assert!(steps(2, 2).is_empty());
}

// Trace: TC-014, FR-004-AC-1
#[test]
fn counterexample_traces_lay_out_propositions_per_time_step() {
    // `F[0,1] p0` and `F[0,1] p1` first differ when only p0 holds at time 0, so
    // the retained trace names p0 at time 0 and nothing at time 1.
    let eventually = |proposition_id: u32| {
        document(
            SemanticProfile::ClosedTraceV1,
            vec![
                proposition(proposition_id),
                Node::new(NodeKind::Future {
                    interval: Interval::new(0, 1).unwrap(),
                    operand: NodeId(0),
                }),
            ],
        )
    };
    let report = check_equivalence(
        &eventually(0),
        &eventually(1),
        "layout",
        ConformanceOptions::default(),
    );
    assert_eq!(report.status, ConformanceStatus::Mismatch);
    assert_eq!(report.counterexample, Some(vec![vec![0], vec![]]));
}

// Trace: TC-066, FR-019-AC-1
#[test]
fn infinite_catalog_rules_name_the_infinite_trace_profile() {
    let catalog = infinite_catalog();
    assert!(!catalog.rules.is_empty());
    for rule in &catalog.rules {
        assert_eq!(
            rule.semantic_profiles,
            vec!["mltl.infinite-trace/v1".to_owned()],
            "{}",
            rule.id
        );
    }
}

// Trace: TC-066, FR-019-AC-1
#[test]
fn record_ceilings_have_their_documented_values() {
    assert_eq!(MAX_INFINITE_REPORT_BYTES, 67_108_864);
    let owner = RecordLimits::OWNER_MAXIMA;
    assert_eq!(owner.document_bytes, 67_108_864);
    assert_eq!(owner.json_depth, 128);
    assert_eq!(owner.string_bytes, 65_536);
}

fn bound_catalog() -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        vec![OwnedSignalDeclaration::new(
            SignalId(11),
            "request_ready".to_owned(),
            SignalDomain::Boolean,
        )],
        vec![PropositionBinding::new(PropositionId(3), SignalId(11))],
    )
    .unwrap()
}

// Trace: TC-066, FR-010-AC-5
#[test]
fn contextual_record_reader_accepts_only_a_record_the_input_reproduces() {
    let input = document(
        SemanticProfile::ClosedTraceV1,
        vec![
            Node::new(NodeKind::True),
            proposition(3),
            Node::new(NodeKind::And {
                left: NodeId(0),
                right: NodeId(1),
            }),
        ],
    );
    let catalog = bound_catalog();
    let report = rewrite_with_context(
        &input,
        "context",
        RewriteOptions::default(),
        "source",
        &catalog,
        None,
    );
    assert_eq!(report.status, RewriteStatus::Normalized);
    let bytes = serde_json::to_vec(&report).unwrap();
    let read = read_with_context(&bytes, &input, &catalog, None, RecordLimits::OWNER_MAXIMA)
        .expect("a record its own input reproduces is admitted");
    assert_eq!(read, report);

    // The same bytes against a different input are not reproduced.
    let other = document(SemanticProfile::ClosedTraceV1, vec![proposition(3)]);
    let refused = read_with_context(&bytes, &other, &catalog, None, RecordLimits::OWNER_MAXIMA)
        .expect_err("a record another input does not reproduce is refused");
    assert_eq!(
        refused.to_string(),
        "re-execution did not reproduce the supplied rewrite report"
    );
}

// Trace: TC-066, FR-010-AC-5
#[test]
fn record_read_errors_render_their_detail() {
    let limits = RecordLimits {
        document_bytes: 4,
        ..RecordLimits::OWNER_MAXIMA
    };
    let error = RewriteReport::from_json_bytes(b"{\"schemaVersion\":1}", limits)
        .expect_err("a record past the byte ceiling is refused");
    let rendered = error.to_string();
    assert!(rendered.contains("exceeds limit 4"), "{rendered}");
}

fn infinite_graph(kinds: Vec<K>, root: u32) -> InfiniteFormulaDocument {
    InfiniteFormulaDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        NodeId(root),
        kinds.into_iter().map(InfiniteNode::new).collect(),
    )
    .unwrap()
}

fn infinite_proposition(id: u32) -> K {
    K::Proposition {
        proposition: PropositionId(id),
    }
}

/// `true & p0`, which one pass rewrites and a second pass confirms.
fn infinite_reducible() -> InfiniteFormulaDocument {
    infinite_graph(
        vec![
            K::True,
            infinite_proposition(0),
            K::And {
                left: NodeId(0),
                right: NodeId(1),
            },
        ],
        2,
    )
}

// Trace: TC-066, FR-019-AC-1
#[test]
fn infinite_rewrite_counts_the_passes_it_ran() {
    let run = |input: &InfiniteFormulaDocument, max_iterations: u32| {
        rewrite_infinite(
            input,
            None,
            RewriteOptions {
                budgets: RewriteBudgets {
                    max_iterations,
                    ..RewriteBudgets::default()
                },
                ..RewriteOptions::default()
            },
            "revision",
            1_000_000,
        )
    };
    let changed = run(&infinite_reducible(), 32);
    assert_eq!(changed.status, RewriteStatus::Normalized);
    assert_eq!(changed.iterations, 2);
    let stable = run(&infinite_graph(vec![infinite_proposition(0)], 0), 32);
    assert_eq!(stable.status, RewriteStatus::Unchanged);
    assert_eq!(stable.iterations, 1);
    // The pass budget is charged per pass that ran.
    let none = run(&infinite_reducible(), 0);
    assert_eq!(
        none.failure,
        Some(InfiniteRewriteFailure::Budget(BudgetKind::Iterations))
    );
    assert_eq!(none.iterations, 0);
    let one = run(&infinite_reducible(), 1);
    assert_eq!(
        one.failure,
        Some(InfiniteRewriteFailure::Budget(BudgetKind::Iterations))
    );
    assert_eq!(one.iterations, 1);
    // Dropping an unreachable node is a change even when no rule fires.
    let dead = run(
        &infinite_graph(vec![infinite_proposition(0), infinite_proposition(1)], 0),
        32,
    );
    assert_eq!(dead.status, RewriteStatus::Normalized);
    assert!(dead.steps.is_empty());
}

// Trace: TC-014, FR-004-AC-1
#[test]
fn counterexample_search_distinguishes_formulas_that_differ_only_at_a_later_step() {
    // `F[0,1] p0` and `F[0,1] p0 | (p1 & !F[1,1] p0 & !p0)` differ only on a trace where p1
    // holds at time 0, p0 holds at neither step and so `F[1,1] p0` is false at time 0.
    let eventually_p0 = document(
        SemanticProfile::ClosedTraceV1,
        vec![
            proposition(0),
            Node::new(NodeKind::Future {
                interval: Interval::new(0, 1).unwrap(),
                operand: NodeId(0),
            }),
        ],
    );
    let widened = document(
        SemanticProfile::ClosedTraceV1,
        vec![
            proposition(0),
            Node::new(NodeKind::Future {
                interval: Interval::new(0, 1).unwrap(),
                operand: NodeId(0),
            }),
            proposition(1),
            Node::new(NodeKind::Future {
                interval: Interval::new(1, 1).unwrap(),
                operand: NodeId(0),
            }),
            Node::new(NodeKind::Not { operand: NodeId(3) }),
            Node::new(NodeKind::Not { operand: NodeId(0) }),
            Node::new(NodeKind::And {
                left: NodeId(4),
                right: NodeId(5),
            }),
            Node::new(NodeKind::And {
                left: NodeId(2),
                right: NodeId(6),
            }),
            Node::new(NodeKind::Or {
                left: NodeId(1),
                right: NodeId(7),
            }),
        ],
    );
    let report = check_equivalence(
        &eventually_p0,
        &widened,
        "later-step",
        ConformanceOptions::default(),
    );
    assert_eq!(report.status, ConformanceStatus::Mismatch);
    assert!(report.counterexample.is_some());
}

// Trace: TC-066, FR-019-AC-1
#[test]
fn infinite_rewrite_reports_the_node_budget_when_the_rewritten_graph_outgrows_the_owner_limit() {
    // `F p0` is shared, so rewriting `!F p0` to `G !p0` keeps `F p0` and adds two nodes to a
    // graph that already holds the owner maximum of 100000. The caller's budget is higher
    // than the owner limit, so the owner refusal is what reports the node budget.
    let leaves = 49_998u32;
    let mut kinds: Vec<K> = (1..=leaves).map(infinite_proposition).collect();
    let mut level: Vec<u32> = (0..leaves).collect();
    while level.len() > 1 {
        let mut next = Vec::new();
        for pair in level.chunks(2) {
            if let [left, right] = pair {
                kinds.push(K::Or {
                    left: NodeId(*left),
                    right: NodeId(*right),
                });
                next.push(u32::try_from(kinds.len() - 1).unwrap());
            } else {
                next.push(pair[0]);
            }
        }
        level = next;
    }
    let tree_root = level[0];
    let at = |kinds: &Vec<K>| u32::try_from(kinds.len()).unwrap();
    let p0 = at(&kinds);
    kinds.push(infinite_proposition(0));
    let eventually = at(&kinds);
    kinds.push(K::Future {
        interval: tl_syntax::TemporalInterval::Closed(Interval::new(0, 1).unwrap()),
        operand: NodeId(p0),
    });
    let negated = at(&kinds);
    kinds.push(K::Not {
        operand: NodeId(eventually),
    });
    let both = at(&kinds);
    kinds.push(K::And {
        left: NodeId(eventually),
        right: NodeId(negated),
    });
    let root = at(&kinds);
    kinds.push(K::Or {
        left: NodeId(tree_root),
        right: NodeId(both),
    });
    assert_eq!(kinds.len(), 100_000);
    let input = infinite_graph(kinds, root);
    let options = RewriteOptions {
        budgets: RewriteBudgets {
            max_nodes: 300_000,
            max_work_units: 1_000_000_000,
            ..RewriteBudgets::default()
        },
        ..RewriteOptions::default()
    };
    let report = rewrite_infinite(&input, None, options, "revision", 10_000_000);
    assert_eq!(
        report.failure,
        Some(InfiniteRewriteFailure::Budget(BudgetKind::Nodes))
    );
}

// Trace: TC-066, FR-019-AC-1
#[test]
fn infinite_rewrite_report_byte_ceiling_is_exact() {
    let input = infinite_reducible();
    let run = |max_report_bytes: usize| {
        rewrite_infinite(
            &input,
            None,
            RewriteOptions::default(),
            "revision",
            max_report_bytes,
        )
    };
    let reference = serde_json::to_vec(&run(1_000_000)).unwrap().len();
    // The record embeds its own ceiling, so find the digit count `d` for which
    // the ceiling equals the record's size.
    let size_without_ceiling = reference - "1000000".len();
    let digits = (1..=8)
        .find(|d| {
            let size = size_without_ceiling + d;
            size.to_string().len() == *d && (size - 1).to_string().len() == *d
        })
        .expect("a self-consistent ceiling exists");
    let exact = size_without_ceiling + digits;
    assert_eq!(run(exact).status, RewriteStatus::Normalized);
    let past = run(exact - 1);
    assert_eq!(past.failure, Some(InfiniteRewriteFailure::ReportBytes));
    assert_eq!(
        past.iterations, 2,
        "both passes ran before the record was found too large"
    );
}

// Trace: TC-066, FR-019-AC-1
#[test]
fn infinite_rewrite_refuses_an_empty_source_revision() {
    let input = infinite_reducible();
    let report = rewrite_infinite(&input, None, RewriteOptions::default(), "", 1_000_000);
    assert_eq!(report.status, RewriteStatus::InvalidInput);
    assert_eq!(
        report.failure,
        Some(InfiniteRewriteFailure::IdentityMismatch)
    );
}

// Trace: TC-066, FR-019-AC-1
#[test]
fn infinite_replay_requires_a_successful_report_from_the_trusted_revision_and_catalog() {
    let input = infinite_reducible();
    let report = rewrite_infinite(
        &input,
        None,
        RewriteOptions::default(),
        "revision",
        1_000_000,
    );
    assert!(replay_infinite(&input, None, &report, "revision"));
    assert!(!replay_infinite(&input, None, &report, "other"));
    let mut foreign_catalog = report.clone();
    foreign_catalog.catalog_sha256 = "not-the-catalog".to_owned();
    assert!(!replay_infinite(&input, None, &foreign_catalog, "revision"));
    let failed = rewrite_infinite(
        &input,
        None,
        RewriteOptions {
            budgets: RewriteBudgets {
                max_iterations: 0,
                ..RewriteBudgets::default()
            },
            ..RewriteOptions::default()
        },
        "revision",
        1_000_000,
    );
    assert!(!replay_infinite(&input, None, &failed, "revision"));
}

// Trace: TC-081, NFR-006-AC-5
#[test]
fn past_equivalence_refuses_when_either_side_is_not_a_past_formula() {
    use tl_mltl::{
        ClockBinding, PastEvaluationLimits, PositionHistoryDocument, PositionObservation,
    };
    use tl_rewrite::{check_past_equivalence, PastConformanceReason, PastEvaluationContext};
    let history = PositionHistoryDocument::new(
        "history:mixed".to_owned(),
        1,
        0,
        0,
        Some(ClockBinding::EventPosition),
        vec![PositionObservation::new(0, vec![PropositionId(0)], None)],
    )
    .unwrap();
    let past = tl_syntax::FormulaDocument::new_v2(
        SemanticProfile::OriginCompleteHistoryV1,
        NodeId(0),
        vec![proposition(0)],
    )
    .unwrap();
    let closed = document(SemanticProfile::ClosedTraceV1, vec![proposition(0)]);
    let context = |formula, formula_id| PastEvaluationContext {
        formula,
        formula_id,
        history: &history,
        anchor: 0,
        proposition_map_id: "map",
    };
    for (original, rewritten) in [(&past, &closed), (&closed, &past)] {
        let report = check_past_equivalence(
            &context(original, "a"),
            &context(rewritten, "b"),
            "mixed",
            PastEvaluationLimits::default(),
        );
        assert_eq!(report.status, ConformanceStatus::NonConclusive);
        assert_eq!(
            report.reason,
            Some(PastConformanceReason::UnsupportedProfile)
        );
    }
}

// Trace: TC-066, FR-019-AC-1
#[test]
fn infinite_rewrite_refuses_premises_bound_to_another_graph() {
    use tl_syntax::FairnessPremisesDocument;
    let input = infinite_reducible();
    let other = infinite_graph(
        vec![
            infinite_proposition(0),
            infinite_proposition(1),
            K::Or {
                left: NodeId(0),
                right: NodeId(1),
            },
        ],
        2,
    );
    let premises = FairnessPremisesDocument::new(
        &other,
        other.content_identity().unwrap(),
        InfiniteClock::EventPosition,
        vec![NodeId(0)],
    )
    .expect("premises over the other graph");
    let report = rewrite_infinite(
        &input,
        Some(&premises),
        RewriteOptions::default(),
        "revision",
        1_000_000,
    );
    assert_eq!(report.status, RewriteStatus::InvalidInput);
    assert_eq!(
        report.failure,
        Some(InfiniteRewriteFailure::IdentityMismatch)
    );
    // The same premises over their own graph are accepted.
    let own = rewrite_infinite(
        &other,
        Some(&premises),
        RewriteOptions::default(),
        "revision",
        1_000_000,
    );
    assert_ne!(own.status, RewriteStatus::InvalidInput);
}

// Trace: TC-081, NFR-006-AC-5
#[test]
fn past_context_identities_are_digests_of_the_whole_context() {
    use tl_mltl::{
        ClockBinding, PastEvaluationLimits, PositionHistoryDocument, PositionObservation,
    };
    use tl_rewrite::{check_past_equivalence, PastEvaluationContext};
    let history = PositionHistoryDocument::new(
        "history:identity".to_owned(),
        1,
        0,
        0,
        Some(ClockBinding::EventPosition),
        vec![PositionObservation::new(0, vec![PropositionId(0)], None)],
    )
    .unwrap();
    let formula = tl_syntax::FormulaDocument::new_v2(
        SemanticProfile::OriginCompleteHistoryV1,
        NodeId(0),
        vec![proposition(0)],
    )
    .unwrap();
    let report = |original_id: &'static str, rewritten_id: &'static str| {
        let context = |formula_id| PastEvaluationContext {
            formula: &formula,
            formula_id,
            history: &history,
            anchor: 0,
            proposition_map_id: "map",
        };
        check_past_equivalence(
            &context(original_id),
            &context(rewritten_id),
            "identity",
            PastEvaluationLimits::default(),
        )
    };
    let same = report("a", "a");
    let hex = |text: &str| {
        text.len() == 64
            && text
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    };
    assert!(
        hex(&same.original_context_identity),
        "{}",
        same.original_context_identity
    );
    assert_eq!(
        same.original_context_identity,
        same.rewritten_context_identity
    );
    let different = report("a", "b");
    assert!(hex(&different.rewritten_context_identity));
    assert_ne!(
        different.original_context_identity,
        different.rewritten_context_identity
    );
    assert_eq!(
        different.original_context_identity,
        same.original_context_identity
    );
}
