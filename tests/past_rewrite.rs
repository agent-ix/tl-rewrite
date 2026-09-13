use proptest::prelude::*;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tl_mltl::{
    evaluate_past, fixed_sample_instant, ClockBinding, ClockSample, ExactNumber,
    PastEvaluationLimits, PastEvaluationRelationInput, PositionHistoryDocument,
    PositionObservation,
};
use tl_rewrite::{
    catalog, check_equivalence, past_catalog, replay, replay_with_context, rewrite,
    rewrite_with_context, BudgetKind, ConformanceOptions, ConformanceReason, ConformanceStatus,
    ProvenanceKind, ReplayStatus, RewriteBudgets, RewriteOptions, RewriteStatus, RuleDisposition,
};
use tl_syntax::{
    FormulaDocument, FormulaSchemaVersion, Interval, Node, NodeId, NodeKind,
    OwnedSignalDeclaration, PropositionBinding, PropositionId, SemanticProfile,
    SignalCatalogDocument, SignalDomain, SignalId, SourceSpan,
};

fn proposition(id: u32) -> Node {
    Node::new(NodeKind::Proposition {
        proposition: PropositionId(id),
    })
}

fn past_document(nodes: Vec<Node>) -> FormulaDocument {
    FormulaDocument::new_v2(
        SemanticProfile::OriginCompleteHistoryV1,
        NodeId(u32::try_from(nodes.len() - 1).unwrap()),
        nodes,
    )
    .unwrap()
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

fn root_kind(document: &FormulaDocument) -> NodeKind {
    document.nodes()[document.root().0 as usize].kind
}

fn event_history(values: &[(bool, bool)]) -> PositionHistoryDocument {
    let observations = values
        .iter()
        .enumerate()
        .map(|(position, (p, q))| {
            let mut propositions = Vec::new();
            if *p {
                propositions.push(PropositionId(0));
            }
            if *q {
                propositions.push(PropositionId(1));
            }
            PositionObservation::new(u64::try_from(position).unwrap(), propositions, None)
        })
        .collect();
    PositionHistoryDocument::new(
        "past-rewrite-event",
        1,
        0,
        u64::try_from(values.len() - 1).unwrap(),
        Some(ClockBinding::EventPosition),
        observations,
    )
    .unwrap()
}

fn fixed_history(values: &[(bool, bool)]) -> PositionHistoryDocument {
    let epoch = ExactNumber::new(-1, 2).unwrap();
    let period = ExactNumber::new(3, 2).unwrap();
    let observations = values
        .iter()
        .enumerate()
        .map(|(position, (p, q))| {
            let position = u64::try_from(position).unwrap();
            let mut propositions = Vec::new();
            if *p {
                propositions.push(PropositionId(0));
            }
            if *q {
                propositions.push(PropositionId(1));
            }
            PositionObservation::new(
                position,
                propositions,
                Some(ClockSample {
                    instant: fixed_sample_instant(epoch, period, position).unwrap(),
                    unit: "ticks".to_owned(),
                }),
            )
        })
        .collect();
    PositionHistoryDocument::new(
        "past-rewrite-fixed",
        1,
        0,
        u64::try_from(values.len() - 1).unwrap(),
        Some(ClockBinding::FixedSample {
            epoch,
            period,
            unit: "ticks".to_owned(),
        }),
        observations,
    )
    .unwrap()
}

fn verdict(document: &FormulaDocument, history: &PositionHistoryDocument, anchor: u64) -> bool {
    evaluate_past(
        document.validate().unwrap(),
        "past-rewrite-formula",
        history,
        anchor,
        "past-rewrite-map",
        1,
        PastEvaluationRelationInput::Original,
        PastEvaluationLimits::default(),
    )
    .unwrap()
    .verdict
}

fn signal_catalog() -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        vec![
            OwnedSignalDeclaration::new(SignalId(10), "p".to_owned(), SignalDomain::Boolean),
            OwnedSignalDeclaration::new(SignalId(11), "q".to_owned(), SignalDomain::Boolean),
        ],
        vec![
            PropositionBinding::new(PropositionId(0), SignalId(10)),
            PropositionBinding::new(PropositionId(1), SignalId(11)),
        ],
    )
    .unwrap()
}

// Trace: TC-046, FR-009-AC-1
#[test]
fn past_catalog_is_profile_exact_and_does_not_mutate_the_future_catalog() {
    let future = catalog();
    let past = past_catalog();
    assert_eq!(future.schema_version, "tl-rewrite.catalog/v1");
    assert_eq!(future.catalog_version, "tl-rewrite-rules/v1");
    assert_eq!(
        future.catalog_sha256,
        "b55705c5903db0680e9f688f87a0b7e9f7322e49f7993af0dfbf82a5e54abdda"
    );
    assert_eq!(past.schema_version, "tl-rewrite.past-catalog/v1");
    assert_eq!(past.catalog_version, "tl-rewrite-past-rules/v1");
    assert_ne!(past.catalog_sha256, future.catalog_sha256);
    assert_eq!(past.rules.len(), 26);
    assert_eq!(
        past.catalog_sha256,
        format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&past.rules).unwrap())
        )
    );
    let ids = past
        .rules
        .iter()
        .map(|rule| rule.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), past.rules.len());

    let past_ids = past
        .rules
        .iter()
        .filter(|rule| rule.id.starts_with("past."))
        .map(|rule| rule.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        past_ids,
        ["past.once.strong-previous", "past.triggered.fold-dual"]
    );
    assert!(past.rules.iter().all(|rule| {
        (rule.id.starts_with("bool.") || rule.id.starts_with("past."))
            && rule.semantic_profiles == ["mltl.origin-complete-history/v1"]
            && rule.revision == 1
            && rule.disposition == RuleDisposition::Enabled
            && rule.provenance.kind == ProvenanceKind::StatedDerivation
            && !rule.precondition.is_empty()
            && !rule.provenance.uri.is_empty()
            && !rule.provenance.locator.is_empty()
            && !rule.provenance.statement.is_empty()
            && rule.exclusion_reason.is_none()
    }));
    assert!(past
        .rules
        .iter()
        .all(|rule| !rule.id.starts_with("temporal.") && !rule.id.starts_with("neg.")));
}

// Trace: TC-047, FR-009-AC-2
#[test]
fn once_one_is_folded_to_strong_previous_with_v2_and_identity_preserved() {
    let span = SourceSpan::new(7, 19).unwrap();
    let input = past_document(vec![
        proposition(0),
        Node::with_span(
            NodeKind::Once {
                interval: Interval::new(1, 1).unwrap(),
                operand: NodeId(0),
            },
            span,
        ),
    ]);
    let report = rewrite(
        &input,
        "once-previous",
        RewriteOptions::default(),
        "rewrite-source",
    );
    assert_eq!(report.status, RewriteStatus::Normalized);
    assert_eq!(report.formula_id, "once-previous");
    assert_eq!(report.engine_source_revision, "rewrite-source");
    assert_eq!(report.semantic_profile, "mltl.origin-complete-history/v1");
    assert_eq!(report.catalog_sha256, past_catalog().catalog_sha256);
    assert_eq!(report.steps.len(), 1);
    assert_eq!(report.steps[0].rule_id, "past.once.strong-previous");
    assert_eq!(report.steps[0].source_span, Some(span));
    let output = report.output.unwrap();
    assert_eq!(output.schema_version(), FormulaSchemaVersion::V2);
    assert_eq!(output.semantic_profile(), input.semantic_profile());
    assert_eq!(
        root_kind(&output),
        NodeKind::StrongPrevious { operand: NodeId(0) }
    );
    assert_eq!(output.nodes()[output.root().0 as usize].span, Some(span));
}

// Trace: TC-047, FR-009-AC-2
#[test]
fn exact_expanded_triggered_dual_folds_but_partial_patterns_do_not() {
    let interval = Interval::new(2, 5).unwrap();
    for admitted in [interval, Interval::new(u32::MAX, u32::MAX).unwrap()] {
        let input = triggered_dual(admitted);
        let report = rewrite(
            &input,
            "triggered-dual",
            RewriteOptions::default(),
            "source",
        );
        assert_eq!(report.status, RewriteStatus::Normalized);
        assert_eq!(report.steps.len(), 1);
        assert_eq!(report.steps[0].rule_id, "past.triggered.fold-dual");
        assert_eq!(
            root_kind(report.output.as_ref().unwrap()),
            NodeKind::Triggered {
                interval: admitted,
                left: NodeId(0),
                right: NodeId(1),
            }
        );
    }

    let partial = past_document(vec![
        proposition(0),
        proposition(1),
        Node::new(NodeKind::Not { operand: NodeId(0) }),
        Node::new(NodeKind::Since {
            interval,
            left: NodeId(2),
            right: NodeId(1),
        }),
        Node::new(NodeKind::Not { operand: NodeId(3) }),
    ]);
    let refused = rewrite(
        &partial,
        "partial-dual",
        RewriteOptions::default(),
        "source",
    );
    assert_eq!(refused.status, RewriteStatus::Unchanged);
    assert!(refused.steps.is_empty());
    assert_eq!(refused.output, Some(partial));
}

// Trace: TC-047, TC-050, FR-009-AC-2, FR-009-AC-5
#[test]
fn unreviewed_past_transformations_remain_unchanged() {
    let interval = Interval::new(0, 0).unwrap();
    let documents = vec![
        once(interval),
        once(Interval::new(u32::MAX, u32::MAX).unwrap()),
        past_document(vec![
            proposition(0),
            Node::new(NodeKind::Historically {
                interval,
                operand: NodeId(0),
            }),
        ]),
        past_document(vec![
            proposition(0),
            Node::new(NodeKind::StrongPrevious { operand: NodeId(0) }),
        ]),
        past_document(vec![
            proposition(0),
            proposition(1),
            Node::new(NodeKind::Since {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            }),
        ]),
        past_document(vec![
            proposition(0),
            proposition(1),
            Node::new(NodeKind::Triggered {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            }),
        ]),
    ];
    for input in documents {
        let report = rewrite(&input, "unreviewed", RewriteOptions::default(), "source");
        assert_eq!(report.status, RewriteStatus::Unchanged);
        assert!(report.steps.is_empty());
        assert_eq!(report.output, Some(input));
    }
}

// Trace: TC-047, FR-009-AC-2
#[test]
fn every_past_operator_traverses_and_rebuilds_rewritten_operands() {
    let interval = Interval::new(2, 4).unwrap();
    let unary = [
        NodeKind::Once {
            interval,
            operand: NodeId(2),
        },
        NodeKind::Historically {
            interval,
            operand: NodeId(2),
        },
        NodeKind::StrongPrevious { operand: NodeId(2) },
    ];
    for root in unary {
        let input = past_document(vec![
            Node::new(NodeKind::True),
            proposition(0),
            Node::new(NodeKind::And {
                left: NodeId(0),
                right: NodeId(1),
            }),
            Node::new(root),
        ]);
        let report = rewrite(
            &input,
            "past-unary-traversal",
            RewriteOptions::default(),
            "source",
        );
        assert_eq!(report.status, RewriteStatus::Normalized);
        assert_eq!(report.steps.len(), 1);
        assert_eq!(report.steps[0].rule_id, "bool.and.true-left");
        assert!(matches!(
            root_kind(report.output.as_ref().unwrap()),
            NodeKind::Once {
                operand: NodeId(0),
                ..
            } | NodeKind::Historically {
                operand: NodeId(0),
                ..
            } | NodeKind::StrongPrevious { operand: NodeId(0) }
        ));
    }

    for root in [
        NodeKind::Since {
            interval,
            left: NodeId(2),
            right: NodeId(1),
        },
        NodeKind::Triggered {
            interval,
            left: NodeId(2),
            right: NodeId(1),
        },
    ] {
        let input = past_document(vec![
            Node::new(NodeKind::True),
            proposition(0),
            Node::new(NodeKind::And {
                left: NodeId(0),
                right: NodeId(1),
            }),
            Node::new(root),
        ]);
        let report = rewrite(
            &input,
            "past-binary-traversal",
            RewriteOptions::default(),
            "source",
        );
        assert_eq!(report.status, RewriteStatus::Normalized);
        assert_eq!(report.steps.len(), 1);
        assert!(matches!(
            root_kind(report.output.as_ref().unwrap()),
            NodeKind::Since {
                left: NodeId(0),
                right: NodeId(0),
                ..
            } | NodeKind::Triggered {
                left: NodeId(0),
                right: NodeId(0),
                ..
            }
        ));
    }
}

// Trace: TC-048, FR-009-AC-3
#[test]
fn reviewed_boolean_rules_apply_inside_the_past_profile() {
    let interval = Interval::new(2, 4).unwrap();
    let input = past_document(vec![
        Node::new(NodeKind::True),
        proposition(0),
        Node::new(NodeKind::Once {
            interval,
            operand: NodeId(1),
        }),
        Node::new(NodeKind::And {
            left: NodeId(0),
            right: NodeId(2),
        }),
    ]);
    let report = rewrite(&input, "past-bool", RewriteOptions::default(), "source");
    assert_eq!(report.status, RewriteStatus::Normalized);
    assert_eq!(report.steps.len(), 1);
    assert_eq!(report.steps[0].rule_id, "bool.and.true-left");
    assert_eq!(
        root_kind(report.output.as_ref().unwrap()),
        NodeKind::Once {
            interval,
            operand: NodeId(0),
        }
    );
}

// Trace: TC-048, FR-009-AC-3
#[test]
fn every_reused_boolean_rule_is_exercised_and_preserves_past_verdicts() {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Manifest {
        cases: Vec<Case>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Case {
        id: String,
        document: Option<serde_json::Value>,
    }

    let manifest: Manifest =
        serde_json::from_slice(include_bytes!("../corpus/rules/manifest.json")).unwrap();
    let expected = past_catalog()
        .rules
        .into_iter()
        .filter(|rule| rule.id.starts_with("bool."))
        .map(|rule| rule.id)
        .collect::<std::collections::BTreeSet<_>>();
    let mut observed = std::collections::BTreeSet::new();
    for case in manifest
        .cases
        .into_iter()
        .filter(|case| case.id.starts_with("bool."))
    {
        let mut value = case.document.expect("enabled Boolean case has a document");
        value["schema_version"] = "tl-syntax.formula/v2".into();
        value["semantic_profile"] = "mltl.origin-complete-history/v1".into();
        let input: FormulaDocument = serde_json::from_value(value).unwrap();
        let report = rewrite(&input, case.id.clone(), RewriteOptions::default(), "source");
        assert_eq!(report.status, RewriteStatus::Normalized, "{}", case.id);
        assert!(
            report.steps.iter().any(|step| step.rule_id == case.id),
            "{}",
            case.id
        );
        let output = report.output.unwrap();
        for values in [
            [(false, false)],
            [(false, true)],
            [(true, false)],
            [(true, true)],
        ] {
            for history in [event_history(&values), fixed_history(&values)] {
                assert_eq!(
                    verdict(&input, &history, 0),
                    verdict(&output, &history, 0),
                    "{}",
                    case.id
                );
            }
        }
        observed.insert(case.id);
    }
    assert_eq!(observed, expected);
}

// Trace: TC-049, FR-009-AC-4
#[test]
fn past_rewrite_and_contextual_replay_bind_every_identity_input() {
    let input = triggered_dual(Interval::new(0, 3).unwrap());
    let report = rewrite(&input, "replay", RewriteOptions::default(), "source");
    assert_eq!(replay(&input, &report).status, ReplayStatus::Verified);
    let wire = serde_json::to_vec(&report).unwrap();
    let decoded: tl_rewrite::RewriteReport = serde_json::from_slice(&wire).unwrap();
    assert_eq!(decoded, report);
    assert_eq!(replay(&input, &decoded).status, ReplayStatus::Verified);

    let mut changed = report.clone();
    changed.catalog_sha256 = "0".repeat(64);
    assert_eq!(replay(&input, &changed).status, ReplayStatus::Mismatch);
    changed = report.clone();
    changed.options.budgets.max_work_units += 1;
    assert_eq!(replay(&input, &changed).status, ReplayStatus::Mismatch);
    changed = report.clone();
    changed.steps[0].after_sha256 = "f".repeat(64);
    assert_eq!(replay(&input, &changed).status, ReplayStatus::Mismatch);
    changed = report.clone();
    changed.output = None;
    assert_eq!(replay(&input, &changed).status, ReplayStatus::Mismatch);
    assert_eq!(
        replay(&once(Interval::new(1, 1).unwrap()), &report).status,
        ReplayStatus::Mismatch
    );

    let catalog = signal_catalog();
    let contextual = rewrite_with_context(
        &input,
        "contextual-past",
        RewriteOptions::default(),
        "source",
        &catalog,
        None,
    );
    assert_eq!(contextual.status, RewriteStatus::Normalized);
    assert_eq!(contextual.catalog_sha256, past_catalog().catalog_sha256);
    let contextual_wire = serde_json::to_vec(&contextual).unwrap();
    let contextual_decoded: tl_rewrite::RewriteReport =
        serde_json::from_slice(&contextual_wire).unwrap();
    assert_eq!(contextual_decoded, contextual);
    assert_eq!(
        replay_with_context(&input, &contextual, &catalog, None).status,
        ReplayStatus::Verified
    );
    let changed_catalog = SignalCatalogDocument::new(
        vec![OwnedSignalDeclaration::new(
            SignalId(10),
            "changed".to_owned(),
            SignalDomain::Boolean,
        )],
        vec![PropositionBinding::new(PropositionId(0), SignalId(10))],
    )
    .unwrap();
    assert_eq!(
        replay_with_context(&input, &contextual, &changed_catalog, None).status,
        ReplayStatus::Mismatch
    );
}

// Trace: TC-050, FR-009-AC-5
#[test]
fn unsupported_profiles_and_budget_failure_return_no_partial_formula() {
    let online = FormulaDocument::new(
        SemanticProfile::OnlinePrefixV1,
        NodeId(0),
        vec![proposition(0)],
    )
    .unwrap();
    let unsupported = rewrite(&online, "online", RewriteOptions::default(), "source");
    assert_eq!(unsupported.status, RewriteStatus::UnsupportedProfile);
    assert!(unsupported.output.is_none());
    assert!(unsupported.steps.is_empty());

    let input = once(Interval::new(1, 1).unwrap());
    let exhausted = rewrite(
        &input,
        "budget",
        RewriteOptions {
            budgets: RewriteBudgets {
                max_rule_applications: 0,
                ..RewriteBudgets::default()
            },
            ..RewriteOptions::default()
        },
        "source",
    );
    assert_eq!(exhausted.status, RewriteStatus::BudgetExhausted);
    assert_eq!(
        exhausted.exhausted_budget,
        Some(BudgetKind::RuleApplications)
    );
    assert!(exhausted.output.is_none());

    assert!(FormulaDocument::new_v2(
        SemanticProfile::OriginCompleteHistoryV1,
        NodeId(1),
        vec![
            proposition(0),
            Node::new(NodeKind::Future {
                interval: Interval::new(0, 1).unwrap(),
                operand: NodeId(0),
            }),
        ],
    )
    .is_err());

    let unknown_profile = serde_json::json!({
        "schema_version": "tl-syntax.formula/v2",
        "semantic_profile": "mltl.unknown/v1",
        "root": 0,
        "nodes": [{ "kind": "true" }]
    });
    assert!(serde_json::from_value::<FormulaDocument>(unknown_profile).is_err());
}

// Trace: TC-051, FR-009-AC-5
#[test]
fn future_only_conformance_api_refuses_past_without_claiming_a_proof() {
    let input = once(Interval::new(1, 1).unwrap());
    let output = rewrite(&input, "past", RewriteOptions::default(), "source")
        .output
        .unwrap();
    let report = check_equivalence(
        &input,
        &output,
        "past-is-not-future",
        ConformanceOptions::default(),
    );
    assert_eq!(report.status, ConformanceStatus::NonConclusive);
    assert_eq!(report.reason, Some(ConformanceReason::UnsupportedProfile));
    assert_eq!(report.catalog_sha256, past_catalog().catalog_sha256);
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 192, ..ProptestConfig::default() })]

    // Trace: TC-052, FR-009-AC-6
    #[test]
    fn every_admitted_past_fold_preserves_event_and_fixed_sample_verdicts(
        values in prop::collection::vec((any::<bool>(), any::<bool>()), 1..18),
        left in 0u32..8,
        right in 0u32..8,
    ) {
        let (start, end) = if left <= right { (left, right) } else { (right, left) };
        let interval = Interval::new(start, end).unwrap();
        let originals = [triggered_dual(interval), once(Interval::new(1, 1).unwrap())];
        let event = event_history(&values);
        let fixed = fixed_history(&values);
        for original in originals {
            let report = rewrite(&original, "property", RewriteOptions::default(), "source");
            prop_assert_eq!(report.status, RewriteStatus::Normalized);
            let rewritten = report.output.unwrap();
            for history in [&event, &fixed] {
                for anchor in 0..values.len() {
                    let anchor = u64::try_from(anchor).unwrap();
                    prop_assert_eq!(verdict(&original, history, anchor), verdict(&rewritten, history, anchor));
                }
            }
        }
    }
}
