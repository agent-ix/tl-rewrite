use tl_oracle::{evaluate_documents, Limits as OracleLimits};
use tl_rewrite::{
    catalog, infinite_catalog, past_catalog, replay_infinite, rewrite_infinite,
    InfiniteRewriteFailure, RewriteOptions, RewriteStatus, RuleDisposition,
};
use tl_syntax::{
    FairnessPremisesDocument, InfiniteClock, InfiniteFormulaDocument, InfiniteNode,
    InfiniteNodeKind as Kind, Interval, LassoTraceDocument, NodeId, PartialValuation, PartialValue,
    PropositionId, SemanticProfile, TemporalInterval, TraceObservation, UnboundedInterval,
    ValuationEntry,
};

fn closed(start: u32, end: u32) -> TemporalInterval {
    TemporalInterval::Closed(Interval::new(start, end).unwrap())
}

fn open(start: u32) -> TemporalInterval {
    TemporalInterval::Unbounded(UnboundedInterval::new(start))
}

fn graph(kinds: Vec<Kind>, root: u32) -> InfiniteFormulaDocument {
    InfiniteFormulaDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        NodeId(root),
        kinds.into_iter().map(InfiniteNode::new).collect(),
    )
    .unwrap()
}

fn fixture(id: &str) -> InfiniteFormulaDocument {
    let p = NodeId(0);
    let q = NodeId(1);
    let yes = NodeId(2);
    let no = NodeId(3);
    let not_p = NodeId(4);
    let not_q = NodeId(5);
    let mut kinds = vec![
        Kind::Proposition {
            proposition: PropositionId(0),
        },
        Kind::Proposition {
            proposition: PropositionId(1),
        },
        Kind::True,
        Kind::False,
        Kind::Not { operand: p },
        Kind::Not { operand: q },
    ];
    let root = match id {
        "bool.not.false" => Kind::Not { operand: no },
        "bool.not.true" => Kind::Not { operand: yes },
        "bool.not.double" => Kind::Not { operand: not_p },
        "bool.and.false-left" => Kind::And { left: no, right: p },
        "bool.and.false-right" => Kind::And { left: p, right: no },
        "bool.and.true-left" => Kind::And {
            left: yes,
            right: p,
        },
        "bool.and.true-right" => Kind::And {
            left: p,
            right: yes,
        },
        "bool.and.idempotent" => Kind::And { left: p, right: p },
        "bool.or.true-left" => Kind::Or {
            left: yes,
            right: p,
        },
        "bool.or.true-right" => Kind::Or {
            left: p,
            right: yes,
        },
        "bool.or.false-left" => Kind::Or { left: no, right: p },
        "bool.or.false-right" => Kind::Or { left: p, right: no },
        "bool.or.idempotent" => Kind::Or { left: p, right: p },
        "bool.implies.false-left" => Kind::Implies { left: no, right: p },
        "bool.implies.true-left" => Kind::Implies {
            left: yes,
            right: p,
        },
        "bool.implies.true-right" => Kind::Implies {
            left: p,
            right: yes,
        },
        "bool.implies.false-right" => Kind::Implies { left: p, right: no },
        "bool.implies.reflexive" => Kind::Implies { left: p, right: p },
        "bool.implies.eliminate" => Kind::Implies { left: p, right: q },
        "bool.equivalent.reflexive" => Kind::Equivalent { left: p, right: p },
        "bool.equivalent.true-left" => Kind::Equivalent {
            left: yes,
            right: p,
        },
        "bool.equivalent.true-right" => Kind::Equivalent {
            left: p,
            right: yes,
        },
        "bool.equivalent.false-left" => Kind::Equivalent { left: no, right: p },
        "bool.equivalent.false-right" => Kind::Equivalent { left: p, right: no },
        "temporal.future.singleton" => Kind::Future {
            interval: closed(0, 0),
            operand: p,
        },
        "temporal.globally.singleton" => Kind::Globally {
            interval: closed(0, 0),
            operand: p,
        },
        "temporal.until.singleton" => Kind::Until {
            interval: closed(0, 0),
            left: p,
            right: q,
        },
        "temporal.release.singleton" => Kind::Release {
            interval: closed(0, 0),
            left: p,
            right: q,
        },
        "temporal.future.false" => Kind::Future {
            interval: open(2),
            operand: no,
        },
        "temporal.future.true" => Kind::Future {
            interval: open(2),
            operand: yes,
        },
        "temporal.globally.false" => Kind::Globally {
            interval: open(2),
            operand: no,
        },
        "temporal.globally.true" => Kind::Globally {
            interval: open(2),
            operand: yes,
        },
        "temporal.until.true-left" => Kind::Until {
            interval: open(1),
            left: yes,
            right: q,
        },
        "temporal.release.false-left" => Kind::Release {
            interval: open(1),
            left: no,
            right: q,
        },
        "past.once.strong-previous" => Kind::Once {
            interval: closed(1, 1),
            operand: p,
        },
        "neg.future.dual"
        | "neg.globally.dual"
        | "neg.until.dual"
        | "neg.release.dual"
        | "past.triggered.fold-dual" => {
            let inner = match id {
                "neg.future.dual" => Kind::Future {
                    interval: open(1),
                    operand: p,
                },
                "neg.globally.dual" => Kind::Globally {
                    interval: open(1),
                    operand: p,
                },
                "neg.until.dual" => Kind::Until {
                    interval: open(1),
                    left: p,
                    right: q,
                },
                "neg.release.dual" => Kind::Release {
                    interval: open(1),
                    left: p,
                    right: q,
                },
                "past.triggered.fold-dual" => Kind::Since {
                    interval: open(1),
                    left: not_p,
                    right: not_q,
                },
                _ => unreachable!(),
            };
            kinds.push(inner);
            Kind::Not { operand: NodeId(6) }
        }
        other => panic!("no fixture for enabled rule {other}"),
    };
    kinds.push(root);
    graph(
        kinds,
        if id.starts_with("neg.") || id == "past.triggered.fold-dual" {
            7
        } else {
            6
        },
    )
}

fn trace(prefix: &[[PartialValue; 2]], loop_cells: &[[PartialValue; 2]]) -> LassoTraceDocument {
    let props = vec![PropositionId(0), PropositionId(1)];
    let row = |position: usize, values: [PartialValue; 2]| TraceObservation {
        position: u32::try_from(position).unwrap(),
        valuation: PartialValuation::new(
            "map".to_owned(),
            &props,
            props
                .iter()
                .zip(values)
                .map(|(id, value)| ValuationEntry {
                    proposition: *id,
                    value,
                })
                .collect(),
        )
        .unwrap(),
    };
    let prefix_rows = prefix
        .iter()
        .copied()
        .enumerate()
        .map(|(i, values)| row(i, values))
        .collect();
    let loop_rows = loop_cells
        .iter()
        .copied()
        .enumerate()
        .map(|(i, values)| row(prefix.len() + i, values))
        .collect();
    LassoTraceDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        "map".to_owned(),
        props,
        prefix_rows,
        loop_rows,
    )
    .unwrap()
}

fn run(
    input: &InfiniteFormulaDocument,
    fairness: Option<&FairnessPremisesDocument>,
) -> tl_rewrite::InfiniteRewriteReport {
    rewrite_infinite(
        input,
        fairness,
        RewriteOptions::default(),
        "test-source",
        1_000_000,
    )
}

/// TC-067, FR-019-AC-1: every enabled catalog identity executes on its fixture.
#[test]
fn tc_067_every_infinite_rule_has_exact_fixture() {
    let infinite = infinite_catalog();
    assert_eq!(infinite.schema_version, "tl-rewrite.infinite-catalog/v1");
    assert_ne!(
        infinite.catalog_sha256,
        tl_rewrite::catalog().catalog_sha256
    );
    assert_ne!(
        infinite.catalog_sha256,
        tl_rewrite::past_catalog().catalog_sha256
    );
    assert_eq!(
        infinite
            .rules
            .iter()
            .filter(|r| r.disposition == RuleDisposition::Excluded)
            .count(),
        2
    );
    for rule in infinite
        .rules
        .iter()
        .filter(|r| r.disposition == RuleDisposition::Enabled)
    {
        let input = fixture(&rule.id);
        let report = run(&input, None);
        assert!(report.succeeded(), "{}: {:?}", rule.id, report.failure);
        assert!(
            report.steps.iter().any(|step| step.rule_id == rule.id),
            "{} did not execute",
            rule.id
        );
        assert!(
            replay_infinite(&input, None, &report, "test-source"),
            "{} did not replay",
            rule.id
        );
    }
    assert_eq!(
        catalog().catalog_sha256,
        "b55705c5903db0680e9f688f87a0b7e9f7322e49f7993af0dfbf82a5e54abdda"
    );
    assert_eq!(past_catalog().rules.len(), 26);
}

/// TC-070, FR-020-AC-1: real rewrites agree with the independent oracle.
#[test]
fn tc_070_every_enabled_rule_agrees_on_complete_and_partial_words() {
    use PartialValue::{Conflicting as C, False as F, Missing as M, True as T};
    let words = [
        trace(&[], &[[F, F]]),
        trace(&[], &[[T, F], [F, T]]),
        trace(&[[T, T]], &[[F, T]]),
        trace(&[], &[[M, T]]),
        trace(&[[C, F]], &[[T, M], [F, T]]),
    ];
    let mut compared = 0;
    for rule in infinite_catalog()
        .rules
        .into_iter()
        .filter(|r| r.disposition == RuleDisposition::Enabled)
    {
        let input = fixture(&rule.id);
        let report = run(&input, None);
        let output = report.output.as_ref().unwrap();
        for word in &words {
            for position in 0..6 {
                let before = evaluate_documents(
                    &input,
                    word,
                    input.root(),
                    &[],
                    position,
                    OracleLimits::default(),
                )
                .unwrap();
                let after = evaluate_documents(
                    output,
                    word,
                    output.root(),
                    &[],
                    position,
                    OracleLimits::default(),
                )
                .unwrap();
                assert_eq!(before, after, "rule={}, position={position}", rule.id);
                compared += 1;
            }
        }
    }
    assert!(compared >= 1_000);
}

/// TC-070, FR-020-AC-1: the independent oracle exposes a wrong infinite rule
/// on both a complete lasso and a partial refinement population.
#[test]
fn tc_070_wrong_infinite_rule_yields_complete_and_partial_counterexamples() {
    let input = graph(
        vec![
            Kind::Proposition {
                proposition: PropositionId(0),
            },
            Kind::Future {
                interval: open(0),
                operand: NodeId(0),
            },
        ],
        1,
    );
    let seeded_wrong = graph(vec![Kind::False], 0);
    for (word, expected) in [
        (
            trace(&[], &[[PartialValue::True, PartialValue::False]]),
            tl_oracle::Verdict::Proved,
        ),
        (
            trace(&[], &[[PartialValue::Missing, PartialValue::False]]),
            tl_oracle::Verdict::Inconclusive,
        ),
    ] {
        let actual =
            evaluate_documents(&input, &word, input.root(), &[], 0, OracleLimits::default())
                .unwrap();
        let wrong = evaluate_documents(
            &seeded_wrong,
            &word,
            seeded_wrong.root(),
            &[],
            0,
            OracleLimits::default(),
        )
        .unwrap();
        assert_eq!(actual.verdict, expected);
        assert_eq!(wrong.verdict, tl_oracle::Verdict::Refuted);
        assert_ne!(actual, wrong);
    }
}

/// TC-067, FR-019-AC-1: an open-upper interval never passes a singleton rule.
#[test]
fn tc_067_unbounded_singleton_is_not_admitted() {
    let input = graph(
        vec![
            Kind::Proposition {
                proposition: PropositionId(0),
            },
            Kind::Future {
                interval: open(0),
                operand: NodeId(0),
            },
        ],
        1,
    );
    let report = run(&input, None);
    assert_eq!(report.status, RewriteStatus::Unchanged);
    assert!(report.steps.is_empty());
    assert_eq!(
        report.output.unwrap().nodes()[1].kind,
        input.nodes()[1].kind
    );
}

/// TC-068, FR-019-AC-2: fairness roots are remapped with the same graph.
#[test]
fn tc_068_fairness_remap_preserves_order_and_identity() {
    let input = graph(
        vec![
            Kind::Proposition {
                proposition: PropositionId(0),
            },
            Kind::True,
            Kind::And {
                left: NodeId(1),
                right: NodeId(0),
            },
        ],
        2,
    );
    let fairness = FairnessPremisesDocument::new(
        &input,
        input.content_identity().unwrap(),
        InfiniteClock::EventPosition,
        vec![NodeId(2)],
    )
    .unwrap();
    let report = run(&input, Some(&fairness));
    assert!(report.succeeded());
    let output = report.output.as_ref().unwrap();
    let remapped = report.output_fairness.as_ref().unwrap();
    assert_eq!(
        remapped.graph_identity(),
        output.content_identity().unwrap()
    );
    assert_eq!(remapped.roots(), &[output.root()]);
    assert_eq!(output.semantic_profile(), SemanticProfile::InfiniteTraceV1);
    assert_eq!(output.clock(), InfiniteClock::EventPosition);
    for value in [
        PartialValue::False,
        PartialValue::True,
        PartialValue::Missing,
        PartialValue::Conflicting,
    ] {
        let word = trace(&[], &[[value, PartialValue::True]]);
        let before = evaluate_documents(
            &input,
            &word,
            input.root(),
            fairness.roots(),
            0,
            OracleLimits::default(),
        );
        let after = evaluate_documents(
            output,
            &word,
            output.root(),
            remapped.roots(),
            0,
            OracleLimits::default(),
        );
        assert_eq!(before, after, "fairness value={value:?}");
    }
    assert!(replay_infinite(
        &input,
        Some(&fairness),
        &report,
        "test-source"
    ));
}

/// TC-069, FR-019-AC-3: collapsing distinct premises refuses without output.
#[test]
fn tc_069_unmappable_fairness_refuses() {
    let input = graph(
        vec![
            Kind::Proposition {
                proposition: PropositionId(0),
            },
            Kind::Proposition {
                proposition: PropositionId(0),
            },
            Kind::And {
                left: NodeId(0),
                right: NodeId(1),
            },
        ],
        2,
    );
    let fairness = FairnessPremisesDocument::new(
        &input,
        input.content_identity().unwrap(),
        InfiniteClock::EventPosition,
        vec![NodeId(0), NodeId(1)],
    )
    .unwrap();
    let report = run(&input, Some(&fairness));
    assert_eq!(
        report.failure,
        Some(InfiniteRewriteFailure::UnmappableFairness)
    );
    assert!(report.output.is_none());
    assert!(report.output_fairness.is_none());
    assert!(report.steps.is_empty());
}

/// TC-069, FR-010-AC-6, FR-019-AC-3: foreign identities refuse before infinite dispatch.
#[test]
fn tc_069_foreign_profile_clock_and_premises_refuse_before_rewrite() {
    let foreign_profile = InfiniteFormulaDocument::new(
        SemanticProfile::ClosedTraceV1,
        InfiniteClock::EventPosition,
        NodeId(0),
        vec![InfiniteNode::new(Kind::True)],
    );
    assert!(foreign_profile.is_err());
    let foreign_clock = LassoTraceDocument::from_selected_identities(
        Some("mltl.infinite-trace/v1"),
        "fixed_sample",
        "map".to_owned(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    assert!(foreign_clock.is_err());

    let input = graph(vec![Kind::False], 0);
    let foreign = graph(vec![Kind::True], 0);
    let premises = FairnessPremisesDocument::new(
        &foreign,
        foreign.content_identity().unwrap(),
        InfiniteClock::EventPosition,
        vec![NodeId(0)],
    )
    .unwrap();
    let report = run(&input, Some(&premises));
    assert_eq!(
        report.failure,
        Some(InfiniteRewriteFailure::IdentityMismatch)
    );
    assert!(report.output.is_none() && report.output_fairness.is_none());
    assert!(report.steps.is_empty());
}

/// TC-071, FR-010-AC-6, FR-020-AC-2: the provider is opt-in and oracle dev-only.
#[test]
fn tc_071_oracle_is_dev_only() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("[features]\ndefault = []\n"));
    assert!(manifest.contains("infinite-trace = [\"tl-mltl/infinite-trace\"]"));
    let production = manifest.split("[dev-dependencies]").next().unwrap();
    let development = manifest.split("[dev-dependencies]").nth(1).unwrap();
    assert!(!production.contains("tl-oracle"));
    assert!(development.contains("https://github.com/agent-ix/tl-oracle.git"));
    assert!(development.contains("80c76669c7967186148d21b4bfa3eeb7d89c2c44"));
    let tree = std::process::Command::new("cargo")
        .args(["tree", "--offline", "-e", "normal", "-p", "tl-oracle"])
        .output()
        .expect("cargo dependency inspection");
    assert!(tree.status.success());
    let tree = String::from_utf8(tree.stdout).unwrap();
    assert!(!tree.contains("tl-mltl v"));
    assert!(!tree.contains("tl-rewrite v"));
}

/// TC-075, NFR-005-AC-1: exact work limits succeed and one-under refuses.
#[test]
fn tc_075_exact_and_one_over_budgets() {
    let input = fixture("temporal.future.true");
    let baseline = run(&input, None);
    assert!(baseline.succeeded());
    let exact = RewriteOptions {
        budgets: tl_rewrite::RewriteBudgets {
            max_iterations: baseline.iterations,
            max_nodes: 7,
            max_rule_applications: u64::try_from(baseline.steps.len()).unwrap(),
            max_work_units: baseline.work_units,
        },
        ..RewriteOptions::default()
    };
    let passing = rewrite_infinite(&input, None, exact, "test-source", 1_000_000);
    assert!(
        passing.succeeded(),
        "exact budget failed: {:?}",
        passing.failure
    );
    for budgets in [
        tl_rewrite::RewriteBudgets {
            max_iterations: exact.budgets.max_iterations - 1,
            ..exact.budgets
        },
        tl_rewrite::RewriteBudgets {
            max_nodes: 0,
            ..exact.budgets
        },
        tl_rewrite::RewriteBudgets {
            max_rule_applications: 0,
            ..exact.budgets
        },
        tl_rewrite::RewriteBudgets {
            max_work_units: exact.budgets.max_work_units - 1,
            ..exact.budgets
        },
    ] {
        let attempt = rewrite_infinite(
            &input,
            None,
            RewriteOptions { budgets, ..exact },
            "test-source",
            1_000_000,
        );
        assert_eq!(attempt.status, RewriteStatus::BudgetExhausted);
        assert!(attempt.output.is_none());
        assert!(attempt.steps.is_empty());
        assert!(!replay_infinite(&input, None, &attempt, "test-source"));
    }
    let tiny_report = rewrite_infinite(&input, None, exact, "test-source", 1);
    assert_eq!(
        tiny_report.failure,
        Some(InfiniteRewriteFailure::ReportBytes)
    );
    assert!(tiny_report.output.is_none());
}

/// TC-075, NFR-005-AC-1: replay binds source, input, catalog and premise roots.
#[test]
fn tc_075_replay_refuses_identity_mutations() {
    let input = graph(
        vec![
            Kind::Proposition {
                proposition: PropositionId(0),
            },
            Kind::True,
            Kind::And {
                left: NodeId(1),
                right: NodeId(0),
            },
        ],
        2,
    );
    let fairness = FairnessPremisesDocument::new(
        &input,
        input.content_identity().unwrap(),
        InfiniteClock::EventPosition,
        vec![NodeId(0)],
    )
    .unwrap();
    let report = run(&input, Some(&fairness));
    assert!(report.succeeded());
    assert!(replay_infinite(
        &input,
        Some(&fairness),
        &report,
        "test-source"
    ));
    assert!(!replay_infinite(
        &input,
        Some(&fairness),
        &report,
        "other-source"
    ));
    assert!(!replay_infinite(&input, None, &report, "test-source"));
    for field in [
        "schema", "input", "catalog", "request", "output", "fairness",
    ] {
        let mut changed = report.clone();
        match field {
            "schema" => changed.schema_version = "foreign".to_owned(),
            "input" => changed.input_identity = "foreign".to_owned(),
            "catalog" => changed.catalog_sha256 = "foreign".to_owned(),
            "request" => changed.request_sha256 = "foreign".to_owned(),
            "output" => changed.output_identity = Some("foreign".to_owned()),
            "fairness" => changed.output_fairness_identity = Some("foreign".to_owned()),
            _ => unreachable!(),
        }
        assert!(
            !replay_infinite(&input, Some(&fairness), &changed, "test-source"),
            "{field}"
        );
    }
    let bytes = serde_json::to_vec(&report).unwrap();
    let parsed = tl_rewrite::InfiniteRewriteReport::from_json_bytes(
        &bytes,
        tl_rewrite::RecordLimits::default(),
    )
    .unwrap();
    assert!(replay_infinite(
        &input,
        Some(&fairness),
        &parsed,
        "test-source"
    ));
}

/// TC-072, FR-020-AC-3: deterministic generated cases use real rewrite and oracle paths.
#[test]
fn tc_072_seeded_rewrite_then_oracle_sweep() {
    const SEED: u64 = 0x5eed_0190_2026_0001;
    const CASES: usize = 256;
    let enabled: Vec<_> = infinite_catalog()
        .rules
        .into_iter()
        .filter(|rule| rule.disposition == RuleDisposition::Enabled)
        .map(|rule| rule.id)
        .collect();
    let mut state = SEED;
    let next = |state: &mut u64| {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *state
    };
    let values = [
        PartialValue::False,
        PartialValue::True,
        PartialValue::Missing,
        PartialValue::Conflicting,
    ];
    let mut exercised = 0;
    for case in 0..CASES {
        let id = &enabled
            [usize::try_from(next(&mut state) % u64::try_from(enabled.len()).unwrap()).unwrap()];
        let basis = fixture(id);
        let mut nodes = basis
            .nodes()
            .iter()
            .map(|node| node.kind)
            .collect::<Vec<_>>();
        let mut root = basis.root();
        for _ in 0..2 {
            let extension = match next(&mut state) % 7 {
                0 => Kind::Not { operand: root },
                1 => Kind::And {
                    left: root,
                    right: NodeId(0),
                },
                2 => Kind::Or {
                    left: NodeId(1),
                    right: root,
                },
                3 => Kind::Future {
                    interval: open(1),
                    operand: root,
                },
                4 => Kind::Historically {
                    interval: open(1),
                    operand: root,
                },
                5 => Kind::Until {
                    interval: closed(1, 3),
                    left: NodeId(0),
                    right: root,
                },
                6 => Kind::Triggered {
                    interval: open(0),
                    left: root,
                    right: NodeId(1),
                },
                _ => unreachable!(),
            };
            root = NodeId(u32::try_from(nodes.len()).unwrap());
            nodes.push(extension);
        }
        let input = graph(nodes, root.0);
        let report = run(&input, None);
        assert!(
            report.succeeded() && !report.steps.is_empty(),
            "seed={SEED}, case={case}, rule={id}"
        );
        let cell = |state: &mut u64| {
            [
                values[usize::try_from(next(state) % 4).unwrap()],
                values[usize::try_from(next(state) % 4).unwrap()],
            ]
        };
        let word = trace(&[cell(&mut state)], &[cell(&mut state), cell(&mut state)]);
        let selected = usize::try_from(next(&mut state) % 10).unwrap();
        let before = evaluate_documents(
            &input,
            &word,
            input.root(),
            &[],
            selected,
            OracleLimits::default(),
        )
        .unwrap_or_else(|error| panic!("seed={SEED}, case={case}, before={error:?}"));
        let output = report.output.as_ref().unwrap();
        let after = evaluate_documents(
            output,
            &word,
            output.root(),
            &[],
            selected,
            OracleLimits::default(),
        )
        .unwrap_or_else(|error| panic!("seed={SEED}, case={case}, after={error:?}"));
        assert_eq!(
            before, after,
            "seed={SEED}, case={case}, rule={id}, position={selected}"
        );
        exercised += 1;
    }
    assert_eq!(exercised, CASES);
}
