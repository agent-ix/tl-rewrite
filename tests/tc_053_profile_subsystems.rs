use tl_mltl::{
    fixed_sample_instant, ClockBinding, ClockSample, ExactNumber, PastEvaluationLimits,
    PositionHistoryDocument, PositionObservation,
};
use tl_rewrite::{
    check_past_equivalence, engine, past_catalog, replay as replay_report,
    report as rewrite_report, rewrite, BudgetKind, ConformanceStatus, PastConformanceReason,
    PastEvaluationContext, RecordLimits, RecordReadErrorCode, ReplayReport, ReplayStatus,
    RewriteBudgets, RewriteOptions, RewriteStatus, TL_MLTL_REVISION, TL_SYNTAX_REVISION,
};
use tl_syntax::{
    FormulaDocument, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile,
};

/// Selects between the two TL-native `ClockBinding` shapes a
/// [`PositionHistoryDocument`] fixture can carry. Before TL-179 this also fed
/// quire-observation authority-view fixtures (`ClockRange`, `Anchor`,
/// `TemporalBoundary`); tl-mltl 0.2.0 dropped the owner-admission machinery
/// that consumed those, so this enum is now scoped to exactly what
/// [`history_document`] still needs.
#[derive(Clone, Copy, Debug)]
enum FixtureClock {
    EventPosition,
    FixedSample,
}

fn past_document(nodes: Vec<Node>) -> FormulaDocument {
    let root = NodeId(u32::try_from(nodes.len() - 1).unwrap());
    FormulaDocument::new_v2(SemanticProfile::OriginCompleteHistoryV1, root, nodes).unwrap()
}

fn proposition(id: u32) -> Node {
    Node::new(NodeKind::Proposition {
        proposition: PropositionId(id),
    })
}

fn once(interval: Interval) -> FormulaDocument {
    past_document(vec![
        proposition(0),
        Node::new(NodeKind::Once {
            interval,
            operand: NodeId(0),
        }),
    ])
}

fn triggered_dual(interval: Interval) -> FormulaDocument {
    past_document(vec![
        proposition(0),
        proposition(1),
        Node::new(NodeKind::Not { operand: NodeId(0) }),
        Node::new(NodeKind::Not { operand: NodeId(1) }),
        Node::new(NodeKind::Since {
            interval,
            left: NodeId(2),
            right: NodeId(3),
        }),
        Node::new(NodeKind::Not { operand: NodeId(4) }),
    ])
}

fn constant(value: bool) -> FormulaDocument {
    past_document(vec![Node::new(if value {
        NodeKind::True
    } else {
        NodeKind::False
    })])
}

fn history_document(
    tag: &str,
    values: &[(bool, bool)],
    clock: FixtureClock,
) -> PositionHistoryDocument {
    let epoch = ExactNumber::new(100, 1).unwrap();
    let period = ExactNumber::new(10, 1).unwrap();
    let observations = values
        .iter()
        .enumerate()
        .map(|(position, (p, q))| {
            let position = u64::try_from(position).unwrap();
            let propositions = [p.then_some(PropositionId(0)), q.then_some(PropositionId(1))]
                .into_iter()
                .flatten()
                .collect();
            PositionObservation::new(
                position,
                propositions,
                match clock {
                    FixtureClock::EventPosition => None,
                    FixtureClock::FixedSample => Some(ClockSample {
                        instant: fixed_sample_instant(epoch, period, position).unwrap(),
                        unit: "nanoseconds".to_owned(),
                    }),
                },
            )
        })
        .collect();
    PositionHistoryDocument::new(
        format!("history:{tag}"),
        1,
        0,
        u64::try_from(values.len() - 1).unwrap(),
        Some(match clock {
            FixtureClock::EventPosition => ClockBinding::EventPosition,
            FixtureClock::FixedSample => ClockBinding::FixedSample {
                epoch,
                period,
                unit: "nanoseconds".to_owned(),
            },
        }),
        observations,
    )
    .unwrap()
}

/// Bundles one formula pair into the two [`PastEvaluationContext`]s
/// `check_past_equivalence` compares, sharing one history, anchor and
/// proposition-map identity. `history` is built by the caller (via
/// [`history_document`]) so it outlives both contexts.
fn context_pair<'a>(
    original: &'a FormulaDocument,
    rewritten: &'a FormulaDocument,
    history: &'a PositionHistoryDocument,
    anchor: u64,
) -> (PastEvaluationContext<'a>, PastEvaluationContext<'a>) {
    (
        PastEvaluationContext {
            formula: original,
            formula_id: "correspondence:original",
            history,
            anchor,
            proposition_map_id: "proposition-map:tc-053",
        },
        PastEvaluationContext {
            formula: rewritten,
            formula_id: "correspondence:rewritten",
            history,
            anchor,
            proposition_map_id: "proposition-map:tc-053",
        },
    )
}

fn record_usage(value: &serde_json::Value) -> (usize, usize) {
    let mut maximum_depth = 0_usize;
    let mut maximum_string = 0_usize;
    let mut pending = vec![(value, 0_usize)];
    while let Some((value, parent_depth)) = pending.pop() {
        match value {
            serde_json::Value::Array(values) => {
                let depth = parent_depth + 1;
                maximum_depth = maximum_depth.max(depth);
                pending.extend(values.iter().map(|value| (value, depth)));
            }
            serde_json::Value::Object(fields) => {
                let depth = parent_depth + 1;
                maximum_depth = maximum_depth.max(depth);
                for (key, value) in fields {
                    maximum_string = maximum_string.max(key.len());
                    pending.push((value, depth));
                }
            }
            serde_json::Value::String(value) => {
                maximum_string = maximum_string.max(value.len());
            }
            _ => {}
        }
    }
    (maximum_depth, maximum_string)
}

// Trace: TC-053, FR-010-AC-1, FR-010-AC-2, FR-010-AC-3
#[test]
fn tc_053_profile_dispatch_owner_admission_and_legacy_bytes_are_preserved() {
    assert_eq!(
        engine::future::SEMANTIC_PROFILE,
        SemanticProfile::ClosedTraceV1
    );
    assert_eq!(
        engine::past::SEMANTIC_PROFILE,
        SemanticProfile::OriginCompleteHistoryV1
    );
    assert_eq!(
        TL_SYNTAX_REVISION,
        "08c23fa319a0ed2cf4535367b3670aa1f2033acd"
    );
    assert_eq!(TL_MLTL_REVISION, "fb9afe042cb34ffc9e3df836e9baaf3fdf0f6cd2");

    let future = FormulaDocument::new(
        SemanticProfile::ClosedTraceV1,
        NodeId(1),
        vec![
            Node::new(NodeKind::True),
            Node::new(NodeKind::Future {
                interval: Interval::new(0, 3).unwrap(),
                operand: NodeId(0),
            }),
        ],
    )
    .unwrap();
    let future_report = rewrite(&future, "future", RewriteOptions::default(), "source");
    assert_eq!(future_report.status, RewriteStatus::Normalized);
    assert_eq!(future_report.steps[0].rule_id, "temporal.future.true");
    assert_eq!(
        future_report.catalog_sha256,
        "b55705c5903db0680e9f688f87a0b7e9f7322e49f7993af0dfbf82a5e54abdda"
    );

    let past = once(Interval::new(1, 1).unwrap());
    let past_report = rewrite(&past, "past", RewriteOptions::default(), "source");
    assert_eq!(past_report.status, RewriteStatus::Normalized);
    assert_eq!(past_report.steps[0].rule_id, "past.once.strong-previous");
    assert_eq!(past_report.catalog_sha256, past_catalog().catalog_sha256);
    let output = past_report.output.as_ref().unwrap();
    let output_bytes = output.canonical_json_bytes().unwrap();
    assert_eq!(
        FormulaDocument::from_json_bytes(&output_bytes, tl_syntax::SyntaxArtifactLimits::default())
            .unwrap(),
        *output
    );

    let engine_source = include_str!("../src/engine/mod.rs");
    let future_source = include_str!("../src/engine/future.rs");
    let past_source = include_str!("../src/engine/past.rs");
    assert!(engine_source.contains("FormulaDocument::from_json_bytes"));
    assert!(engine_source.contains("future::apply_first"));
    assert!(engine_source.contains("past::apply_first"));
    assert!(future_source.contains("neg.future.dual"));
    assert!(future_source.contains("temporal.release.false-left"));
    assert!(!future_source.contains("past.once.strong-previous"));
    assert!(!future_source.contains("past.triggered.fold-dual"));
    assert!(past_source.contains("past.once.strong-previous"));
    assert!(past_source.contains("past.triggered.fold-dual"));
    assert!(!past_source.contains("neg.future.dual"));
    assert!(!past_source.contains("temporal.release.false-left"));
}

// Trace: TC-053, FR-010-AC-3, FR-010-AC-5
#[test]
fn tc_053_report_replay_and_all_work_limits_are_exact_and_fail_closed() {
    let input = once(Interval::new(1, 1).unwrap());
    let report = rewrite(&input, "strict", RewriteOptions::default(), "source");
    let bytes = serde_json::to_vec(&report).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let (json_depth, string_bytes) = record_usage(&value);
    let exact = RecordLimits {
        document_bytes: bytes.len(),
        json_depth,
        string_bytes,
    };
    assert_eq!(rewrite_report::read(&bytes, &input, exact).unwrap(), report);
    assert_eq!(
        rewrite_report::read(
            &bytes,
            &input,
            RecordLimits {
                document_bytes: bytes.len() - 1,
                ..RecordLimits::default()
            }
        )
        .unwrap_err()
        .code(),
        RecordReadErrorCode::ResourceLimit
    );
    assert_eq!(
        rewrite_report::read(
            &bytes,
            &input,
            RecordLimits {
                json_depth: json_depth - 1,
                ..exact
            }
        )
        .unwrap_err()
        .code(),
        RecordReadErrorCode::ResourceLimit
    );
    assert_eq!(
        rewrite_report::read(
            &bytes,
            &input,
            RecordLimits {
                string_bytes: string_bytes - 1,
                ..exact
            }
        )
        .unwrap_err()
        .code(),
        RecordReadErrorCode::ResourceLimit
    );

    let mut trailing = bytes.clone();
    trailing.push(b' ');
    assert_eq!(
        rewrite_report::read(&trailing, &input, RecordLimits::default())
            .unwrap_err()
            .code(),
        RecordReadErrorCode::NonCanonical
    );
    let duplicate = String::from_utf8(bytes.clone())
        .unwrap()
        .replacen('{', "{\"schemaVersion\":\"tl-rewrite.report/v1\",", 1)
        .into_bytes();
    assert_eq!(
        rewrite_report::read(&duplicate, &input, RecordLimits::default())
            .unwrap_err()
            .code(),
        RecordReadErrorCode::NonCanonical
    );
    let mut unknown = value.clone();
    unknown["unknown"] = serde_json::Value::Bool(true);
    assert_eq!(
        rewrite_report::read(
            &serde_json::to_vec(&unknown).unwrap(),
            &input,
            RecordLimits::default()
        )
        .unwrap_err()
        .code(),
        RecordReadErrorCode::Malformed
    );
    let reordered = serde_json::to_vec(&value).unwrap();
    assert_ne!(reordered, bytes);
    assert_eq!(
        rewrite_report::read(&reordered, &input, RecordLimits::default())
            .unwrap_err()
            .code(),
        RecordReadErrorCode::NonCanonical
    );
    let mut changed = report.clone();
    changed.catalog_sha256 = "0".repeat(64);
    let changed_bytes = serde_json::to_vec(&changed).unwrap();
    assert_eq!(
        rewrite_report::read(&changed_bytes, &input, RecordLimits::default())
            .unwrap_err()
            .code(),
        RecordReadErrorCode::ExpectedMismatch
    );

    let replay = replay_report(&input, &report);
    assert_eq!(replay.status, ReplayStatus::Verified);
    let replay_bytes = serde_json::to_vec(&replay).unwrap();
    let replay_value: serde_json::Value = serde_json::from_slice(&replay_bytes).unwrap();
    let (replay_depth, replay_string_bytes) = record_usage(&replay_value);
    let exact_replay_limits = RecordLimits {
        document_bytes: replay_bytes.len(),
        json_depth: replay_depth,
        string_bytes: replay_string_bytes,
    };
    assert_eq!(
        ReplayReport::from_json_bytes(&replay_bytes, exact_replay_limits).unwrap(),
        replay
    );
    assert_eq!(
        ReplayReport::from_json_bytes(
            &replay_bytes,
            RecordLimits {
                document_bytes: replay_bytes.len() - 1,
                ..RecordLimits::default()
            }
        )
        .unwrap_err()
        .code(),
        RecordReadErrorCode::ResourceLimit
    );
    assert_eq!(
        ReplayReport::from_json_bytes(
            &replay_bytes,
            RecordLimits {
                json_depth: replay_depth - 1,
                ..exact_replay_limits
            }
        )
        .unwrap_err()
        .code(),
        RecordReadErrorCode::ResourceLimit
    );
    assert_eq!(
        ReplayReport::from_json_bytes(
            &replay_bytes,
            RecordLimits {
                string_bytes: replay_string_bytes - 1,
                ..exact_replay_limits
            }
        )
        .unwrap_err()
        .code(),
        RecordReadErrorCode::ResourceLimit
    );
    let mut replay_trailing = replay_bytes.clone();
    replay_trailing.push(b' ');
    assert_eq!(
        ReplayReport::from_json_bytes(&replay_trailing, RecordLimits::default())
            .unwrap_err()
            .code(),
        RecordReadErrorCode::NonCanonical
    );
    let replay_duplicate = String::from_utf8(replay_bytes.clone())
        .unwrap()
        .replacen('{', "{\"schemaVersion\":\"tl-rewrite.replay/v1\",", 1)
        .into_bytes();
    assert_eq!(
        ReplayReport::from_json_bytes(&replay_duplicate, RecordLimits::default())
            .unwrap_err()
            .code(),
        RecordReadErrorCode::NonCanonical
    );
    let mut replay_unknown = replay_value.clone();
    replay_unknown["unknown"] = serde_json::Value::Bool(true);
    assert_eq!(
        ReplayReport::from_json_bytes(
            &serde_json::to_vec(&replay_unknown).unwrap(),
            RecordLimits::default()
        )
        .unwrap_err()
        .code(),
        RecordReadErrorCode::Malformed
    );
    let replay_reordered = serde_json::to_vec(&replay_value).unwrap();
    assert_ne!(replay_reordered, replay_bytes);
    assert_eq!(
        ReplayReport::from_json_bytes(&replay_reordered, RecordLimits::default())
            .unwrap_err()
            .code(),
        RecordReadErrorCode::NonCanonical
    );

    let boundaries = [
        (
            RewriteBudgets {
                max_iterations: 0,
                ..RewriteBudgets::default()
            },
            BudgetKind::Iterations,
        ),
        (
            RewriteBudgets {
                max_nodes: 1,
                ..RewriteBudgets::default()
            },
            BudgetKind::Nodes,
        ),
        (
            RewriteBudgets {
                max_rule_applications: 0,
                ..RewriteBudgets::default()
            },
            BudgetKind::RuleApplications,
        ),
        (
            RewriteBudgets {
                max_work_units: 0,
                ..RewriteBudgets::default()
            },
            BudgetKind::WorkUnits,
        ),
    ];
    for (budgets, expected) in boundaries {
        let observed = rewrite(
            &input,
            "limit",
            RewriteOptions {
                budgets,
                ..RewriteOptions::default()
            },
            "source",
        );
        assert_eq!(observed.status, RewriteStatus::BudgetExhausted);
        assert_eq!(observed.exhausted_budget, Some(expected));
        assert!(observed.output.is_none());
    }
}

// Trace: TC-053, FR-010-AC-2, FR-010-AC-4
#[test]
fn tc_053_past_owner_equivalence_covers_both_clocks_all_anchors_and_wrong_operators() {
    let values = [(false, true), (true, false), (false, true), (true, true)];
    let correct_pairs = [
        {
            let original = once(Interval::new(1, 1).unwrap());
            let rewritten = rewrite(&original, "once", RewriteOptions::default(), "source")
                .output
                .unwrap();
            (original, rewritten)
        },
        {
            let original = triggered_dual(Interval::new(0, 2).unwrap());
            let rewritten = rewrite(&original, "triggered", RewriteOptions::default(), "source")
                .output
                .unwrap();
            (original, rewritten)
        },
    ];
    for clock in [FixtureClock::EventPosition, FixtureClock::FixedSample] {
        for anchor in 0..u64::try_from(values.len()).unwrap() {
            for (index, (original, rewritten)) in correct_pairs.iter().enumerate() {
                let tag = format!("correct-{clock:?}-{anchor}-{index}");
                let history = history_document(&tag, &values, clock);
                let (original_context, rewritten_context) =
                    context_pair(original, rewritten, &history, anchor);
                let report = check_past_equivalence(
                    &original_context,
                    &rewritten_context,
                    tag,
                    PastEvaluationLimits::default(),
                );
                assert_eq!(report.status, ConformanceStatus::Equivalent);
                assert_eq!(report.reason, None);
                assert_eq!(report.syntax_revision, TL_SYNTAX_REVISION);
                assert_eq!(report.evaluator_revision, TL_MLTL_REVISION);
                assert!(report.original_result_identity.is_some());
                assert!(report.rewritten_result_identity.is_some());
                assert!(report.limitation.contains("does not prove a universal"));
            }
        }
    }

    let interval = Interval::new(0, 1).unwrap();
    let wrong_pairs = [
        ("once", once(interval), constant(true), 0),
        (
            "historically",
            past_document(vec![
                proposition(0),
                Node::new(NodeKind::Historically {
                    interval,
                    operand: NodeId(0),
                }),
            ]),
            constant(true),
            1,
        ),
        (
            "strong-previous",
            past_document(vec![
                proposition(0),
                Node::new(NodeKind::StrongPrevious { operand: NodeId(0) }),
            ]),
            constant(true),
            0,
        ),
        (
            "since",
            past_document(vec![
                Node::new(NodeKind::True),
                proposition(0),
                Node::new(NodeKind::Since {
                    interval: Interval::new(0, 0).unwrap(),
                    left: NodeId(0),
                    right: NodeId(1),
                }),
            ]),
            constant(true),
            0,
        ),
        (
            "triggered",
            past_document(vec![
                Node::new(NodeKind::True),
                proposition(1),
                Node::new(NodeKind::Triggered {
                    interval: Interval::new(0, 0).unwrap(),
                    left: NodeId(0),
                    right: NodeId(1),
                }),
            ]),
            constant(false),
            0,
        ),
    ];
    for clock in [FixtureClock::EventPosition, FixtureClock::FixedSample] {
        for (operator, original, wrong, anchor) in &wrong_pairs {
            let tag = format!("wrong-{clock:?}-{operator}");
            let history = history_document(&tag, &values, clock);
            let (original_context, wrong_context) =
                context_pair(original, wrong, &history, *anchor);
            let report = check_past_equivalence(
                &original_context,
                &wrong_context,
                tag,
                PastEvaluationLimits::default(),
            );
            assert_eq!(report.status, ConformanceStatus::Mismatch, "{operator}");
            assert_eq!(report.reason, None);
        }
    }

    let original = once(Interval::new(1, 1).unwrap());
    let rewritten = rewrite(&original, "once", RewriteOptions::default(), "source")
        .output
        .unwrap();

    // Both sides of a context-mismatch comparison must share one history,
    // anchor and proposition map (see `matching_past_context` in
    // src/equivalence.rs); here only the anchor differs.
    let context_mismatch_history =
        history_document("context-original", &values, FixtureClock::EventPosition);
    let original_at_anchor_zero = PastEvaluationContext {
        formula: &original,
        formula_id: "correspondence:original",
        history: &context_mismatch_history,
        anchor: 0,
        proposition_map_id: "proposition-map:tc-053",
    };
    let rewritten_at_anchor_one = PastEvaluationContext {
        formula: &rewritten,
        formula_id: "correspondence:rewritten",
        history: &context_mismatch_history,
        anchor: 1,
        proposition_map_id: "proposition-map:tc-053",
    };
    let refused = check_past_equivalence(
        &original_at_anchor_zero,
        &rewritten_at_anchor_one,
        "context-mismatch",
        PastEvaluationLimits::default(),
    );
    assert_eq!(refused.status, ConformanceStatus::NonConclusive);
    assert_eq!(refused.reason, Some(PastConformanceReason::ContextMismatch));

    let resource_history =
        history_document("resource-nonvalue", &values, FixtureClock::EventPosition);
    let (original_context, rewritten_context) =
        context_pair(&original, &rewritten, &resource_history, 1);
    let nonvalue = check_past_equivalence(
        &original_context,
        &rewritten_context,
        "resource-nonvalue",
        PastEvaluationLimits {
            max_steps: 0,
            ..PastEvaluationLimits::default()
        },
    );
    assert_eq!(nonvalue.status, ConformanceStatus::NonConclusive);
    // Before tl-mltl 0.2.0 (TL-179) an exhausted step budget still produced a
    // completed owner result with a non-final/non-Boolean truth
    // (`TemporalTruth::Unavailable`), reported here as a dedicated
    // `NonBooleanResult` reason. `tl_mltl::past::evaluate_past` (the TL-179
    // replacement) has no such intermediate state for the past lane: it
    // either returns a definite Boolean verdict or a typed
    // `PastEvaluationError` (here `StepLimitExceeded`). A step-limit refusal
    // now reports the same way as any other evaluator refusal.
    assert_eq!(
        nonvalue.reason,
        Some(PastConformanceReason::OriginalEvaluatorError)
    );
    assert!(nonvalue.original_result_identity.is_none());
    assert!(nonvalue.rewritten_result_identity.is_none());

    let one_node = constant(true);
    let two_nodes = once(Interval::new(1, 1).unwrap());
    let refusal_history = history_document("owner-refusal", &values, FixtureClock::EventPosition);
    let (short_context, long_context) = context_pair(&one_node, &two_nodes, &refusal_history, 1);
    // `max_formula_nodes` no longer exists on `PastEvaluationLimits` (tl-mltl
    // 0.2.0 bounds past evaluation by recursion depth, not node count).
    // `max_recursion_depth: 0` reproduces the same asymmetry: `one_node` is a
    // single leaf evaluated at depth 0 and still succeeds, while `two_nodes`
    // (`once`) must recurse into its operand at depth 1 and is refused.
    let rewritten_refused = check_past_equivalence(
        &short_context,
        &long_context,
        "rewritten-owner-refusal",
        PastEvaluationLimits {
            max_recursion_depth: 0,
            ..PastEvaluationLimits::default()
        },
    );
    assert_eq!(
        rewritten_refused.reason,
        Some(PastConformanceReason::RewrittenEvaluatorError)
    );
    assert!(rewritten_refused.original_result_identity.is_some());
    assert!(rewritten_refused.rewritten_result_identity.is_none());

    let original_refused = check_past_equivalence(
        &long_context,
        &short_context,
        "original-owner-refusal",
        PastEvaluationLimits {
            max_recursion_depth: 0,
            ..PastEvaluationLimits::default()
        },
    );
    assert_eq!(
        original_refused.reason,
        Some(PastConformanceReason::OriginalEvaluatorError)
    );
    assert!(original_refused.original_result_identity.is_none());
    assert!(original_refused.rewritten_result_identity.is_none());
}
