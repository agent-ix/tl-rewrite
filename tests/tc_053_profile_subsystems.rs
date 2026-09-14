use quire_observation::authority::{
    self, AuthoritySelection, Context, History, Limits as ObservationLimits, OpenClosed,
    SubjectSelection, TemporalBoundary,
};
use quire_observation::{
    admit, AdmissionOutcome, AdmissionRequest, AdmittedRecord, Anchor, ClockRange, Digest,
    Identity, Member, ObservationBinding, PackageSelection, ProducerSelection,
    QualifiedObservation, ResourceLimits, ScopeKind, ScopeSelection, Subject, SubjectKind,
    ValueState, Visibility, NATIVE_LINKED_PACKAGE_FORMAT, PRODUCER_INTERFACE_VERSION,
};
use tl_mltl::past::history;
use tl_mltl::wire::{request, OwnerLimits};
use tl_mltl::{
    fixed_sample_instant, ClockBinding, ClockSample, ExactNumber, PositionHistoryDocument,
    PositionObservation,
};
use tl_rewrite::{
    check_past_equivalence, engine, past_catalog, replay as replay_report,
    report as rewrite_report, rewrite, BudgetKind, ConformanceStatus, PastConformanceReason,
    RecordLimits, RecordReadErrorCode, ReplayReport, ReplayStatus, RewriteBudgets, RewriteOptions,
    RewriteStatus, TL_MLTL_REVISION, TL_SYNTAX_REVISION,
};
use tl_syntax::{
    FormulaDocument, Interval, Node, NodeId, NodeKind, PropositionEntry, PropositionId,
    PropositionMapDocument, SemanticProfile,
};

#[derive(Clone, Copy, Debug)]
enum FixtureClock {
    EventPosition,
    FixedSample,
}

impl FixtureClock {
    const fn identity(self) -> &'static str {
        match self {
            Self::EventPosition => "clock:event-position",
            Self::FixedSample => "clock:fixed-sample",
        }
    }

    const fn range(self, positions: u64) -> ClockRange {
        match self {
            Self::EventPosition => ClockRange::EventPosition {
                start: 0,
                end_exclusive: positions,
            },
            Self::FixedSample => ClockRange::FixedSample {
                start: 0,
                end_exclusive: positions,
                epoch_nanos: 100,
                period_nanos: 10,
            },
        }
    }

    const fn anchor(self) -> Anchor {
        match self {
            Self::EventPosition => Anchor::EventPosition(0),
            Self::FixedSample => Anchor::FixedSample {
                index: 0,
                epoch_nanos: 100,
                period_nanos: 10,
            },
        }
    }

    const fn boundary(self, positions: u64) -> TemporalBoundary {
        match self {
            Self::EventPosition => TemporalBoundary::EventPosition {
                lower: 0,
                upper_inclusive: positions - 1,
                carrier_end_exclusive: positions,
                watermark: positions,
            },
            Self::FixedSample => TemporalBoundary::FixedSample {
                lower: 0,
                upper_inclusive: positions - 1,
                carrier_end_exclusive: positions,
                watermark: positions,
            },
        }
    }
}

struct OwnerViews {
    clock: authority::clock::View,
    progress: authority::progress::View,
    closure: authority::closure::View,
    completeness: authority::completeness::View,
    availability: authority::availability::View,
}

fn identity(value: impl Into<String>) -> Identity {
    Identity::new(value)
}

fn digest(value: u8) -> Digest {
    Digest::new([value; 32])
}

fn qualified_observation(
    tag: &str,
    positions: u64,
    clock: FixtureClock,
) -> Box<QualifiedObservation> {
    let subject = Subject {
        kind: SubjectKind::Order,
        identity: identity(format!("order:{tag}")),
    };
    let mut request = AdmissionRequest {
        package: PackageSelection {
            format: NATIVE_LINKED_PACKAGE_FORMAT.to_owned(),
            identity: identity(format!("package:{tag}")),
            revision: identity("1"),
            digest: digest(1),
        },
        producer: ProducerSelection {
            interface_version: PRODUCER_INTERFACE_VERSION.to_owned(),
            document_identity: identity(format!("producer:{tag}")),
            document_digest: digest(2),
            model_identity: identity(format!("model:{tag}")),
            configuration_identity: identity(format!("configuration:{tag}")),
            configuration_digest: digest(3),
        },
        binding: ObservationBinding {
            identity: identity(format!("binding:{tag}")),
            source_identity: identity(format!("source:{tag}")),
            schema_identity: identity(format!("schema:{tag}")),
            signal_identity: identity("signal:p"),
            trigger_identity: identity(format!("trigger:{tag}")),
            unit: identity("boolean"),
            subject_kind: SubjectKind::Order,
            required: true,
        },
        expected_subject: subject.clone(),
        relationships: Vec::new(),
        required_relationships: Vec::new(),
        scope: ScopeSelection {
            population_identity: identity("unsealed-population"),
            membership_rule_identity: identity(format!("membership:{tag}")),
            membership_digest: digest(4),
            membership_document: Vec::new(),
            required_member_identities: vec![identity(format!("member:{tag}"))],
            observation_sources: vec![identity(format!("source:{tag}"))],
            completeness_dependencies: vec![identity(format!("completeness:{tag}"))],
            progress_dependencies: vec![identity(format!("progress:{tag}"))],
            clock_identity: identity(clock.identity()),
            clock_revision: identity("1"),
            membership_complete: true,
            closure_identity: Some(identity(format!("closure:{tag}"))),
            closure_digest: Some(digest(5)),
            kind: ScopeKind::Snapshot {
                snapshot_identity: identity(format!("snapshot:{tag}")),
            },
            range: clock.range(positions),
            members: vec![Member {
                object_identity: identity(format!("member:{tag}")),
                record_identity: identity("unsealed-record"),
                anchor: clock.anchor(),
            }],
        },
        records: vec![AdmittedRecord {
            identity: identity("unsealed-record"),
            binding_identity: identity(format!("binding:{tag}")),
            source_identity: identity(format!("source:{tag}")),
            schema_identity: identity(format!("schema:{tag}")),
            subject,
            signal_identity: identity("signal:p"),
            trigger_identity: identity(format!("trigger:{tag}")),
            unit: identity("boolean"),
            value: ValueState::Present {
                value_type: identity("boolean"),
                canonical_value: "true".to_owned(),
            },
            visibility: Visibility::External,
            anchor: clock.anchor(),
            event_time_nanos: 0,
            ingestion_time_nanos: 1,
            causal_relationship_identity: None,
            clock_identity: identity(clock.identity()),
            clock_revision: identity("1"),
            clock_uncertainty_nanos: 0,
        }],
        limits: ResourceLimits {
            max_records: 1,
            max_members: 1,
            max_relationships: 0,
            max_required_relationships: 0,
        },
    };
    let record_identity = authority::observation::record_identity(
        &request.records[0],
        ObservationLimits::owner_max(),
    )
    .unwrap();
    request.records[0].identity = record_identity.clone();
    request.scope.members[0].record_identity = record_identity;
    authority::population::assign_request_identities(&mut request, ObservationLimits::owner_max())
        .unwrap();
    match admit(request) {
        AdmissionOutcome::Available { observation } => observation,
        other => panic!("observation fixture admission failed: {other:?}"),
    }
}

fn owner_views(tag: &str, positions: u64, fixture_clock: FixtureClock) -> OwnerViews {
    let qualified = qualified_observation(tag, positions, fixture_clock);
    let authority = AuthoritySelection {
        definition_identity: identity(format!("definition:{tag}")),
        definition_revision: identity("1"),
        definition_digest: digest(9),
    };
    let subject = SubjectSelection {
        scope_identity: identity(format!("snapshot:{tag}")),
        population_identity: qualified.scope().population_identity.clone(),
    };
    let context = Context::new(History::batch(&qualified), &authority, &subject, 1, None);
    let clock_selection =
        authority::clock::Selection::new(identity(fixture_clock.identity()), identity("1"));
    let clock_document =
        authority::clock::derive(context, &clock_selection, ObservationLimits::owner_max())
            .unwrap();
    let clock = authority::clock::read(
        clock_document.bytes(),
        context,
        &clock_selection,
        ObservationLimits::owner_max(),
    )
    .unwrap();
    let progress_selection = authority::progress::Selection::new(
        clock_selection,
        vec![identity(format!("source:{tag}"))],
        fixture_clock.boundary(positions),
        OpenClosed::Closed,
        identity(format!("trigger:{tag}")),
        authority::observation::CutoffSelection::new(
            identity(fixture_clock.identity()),
            identity("1"),
            2,
            authority::observation::CutoffRule::IngestionTimeAtOrBefore,
            identity("1"),
        ),
        identity(format!("restoration:{tag}")),
    );
    let progress_document =
        authority::progress::derive(context, &progress_selection, ObservationLimits::owner_max())
            .unwrap();
    let progress = authority::progress::read(
        progress_document.bytes(),
        context,
        &progress_selection,
        ObservationLimits::owner_max(),
    )
    .unwrap();
    let closure_selection = authority::closure::Selection::new(
        identity(fixture_clock.identity()),
        identity("1"),
        vec![identity(format!("source:{tag}"))],
        fixture_clock.boundary(positions),
        OpenClosed::Closed,
    );
    let closure_document =
        authority::closure::derive(context, &closure_selection, ObservationLimits::owner_max())
            .unwrap();
    let closure = authority::closure::read(
        closure_document.bytes(),
        context,
        &closure_selection,
        ObservationLimits::owner_max(),
    )
    .unwrap();
    let completeness_selection = authority::completeness::Selection::new(
        identity(format!("boundary:{tag}")),
        vec![authority::completeness::Fact::new(
            identity(format!("member:{tag}")),
            Some(qualified.records()[0].identity.clone()),
            authority::completeness::FactStatus::Available,
        )],
    );
    let completeness_document = authority::completeness::derive(
        context,
        &completeness_selection,
        ObservationLimits::owner_max(),
    )
    .unwrap();
    let completeness = authority::completeness::read(
        completeness_document.bytes(),
        context,
        &completeness_selection,
        ObservationLimits::owner_max(),
    )
    .unwrap();
    let required = vec![identity(format!("required-result:{tag}"))];
    let availability_selection = authority::availability::Selection::new(
        required.clone(),
        required,
        authority::availability::DependencyState::Available,
        authority::availability::DependencyState::Available,
    );
    let availability_document = authority::availability::derive(
        context,
        &availability_selection,
        ObservationLimits::owner_max(),
    )
    .unwrap();
    let availability = authority::availability::read(
        availability_document.bytes(),
        context,
        &availability_selection,
        ObservationLimits::owner_max(),
    )
    .unwrap();
    OwnerViews {
        clock,
        progress,
        closure,
        completeness,
        availability,
    }
}

fn proposition_map() -> PropositionMapDocument {
    PropositionMapDocument::new(vec![
        PropositionEntry {
            id: PropositionId(0),
            name: "p".to_owned(),
        },
        PropositionEntry {
            id: PropositionId(1),
            name: "q".to_owned(),
        },
    ])
    .unwrap()
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

fn admit_history(document: &PositionHistoryDocument) -> history::ValidatedPositionHistory {
    let derived = history::derive(document, OwnerLimits::default()).unwrap();
    history::read(derived.bytes(), document, OwnerLimits::default()).unwrap()
}

fn admit_request(
    formula: &FormulaDocument,
    propositions: &PropositionMapDocument,
    history: &history::ValidatedPositionHistory,
    views: &OwnerViews,
    anchor: u64,
    correspondence: &str,
) -> request::ValidatedTemporalRequest {
    let input = request::RequestInput {
        formula,
        proposition_map: propositions,
        input: request::TemporalInput::Past(history),
        clock: &views.clock,
        subject_identity: "native-subject:rewrite-equivalence",
        correspondence_identity: correspondence,
        anchor,
        observations: request::ObservationInputs {
            decision_scope_progress: &views.progress,
            decision_scope_closure: &views.closure,
            surrounding_execution_progress: &views.progress,
            surrounding_execution_closure: &views.closure,
            completeness: &views.completeness,
            availability: &views.availability,
        },
    };
    let document = request::derive(input, OwnerLimits::default()).unwrap();
    request::read(document.bytes(), input, OwnerLimits::default()).unwrap()
}

fn owner_pair(
    original: &FormulaDocument,
    rewritten: &FormulaDocument,
    values: &[(bool, bool)],
    clock: FixtureClock,
    anchor: u64,
    tag: &str,
) -> (
    request::ValidatedTemporalRequest,
    request::ValidatedTemporalRequest,
) {
    let propositions = proposition_map();
    let history_value = history_document(tag, values, clock);
    let history = admit_history(&history_value);
    let views = owner_views(tag, u64::try_from(values.len()).unwrap(), clock);
    (
        admit_request(
            original,
            &propositions,
            &history,
            &views,
            anchor,
            "correspondence:original",
        ),
        admit_request(
            rewritten,
            &propositions,
            &history,
            &views,
            anchor,
            "correspondence:rewritten",
        ),
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
        "842d82553f045eb69a7f38745756d968254fc25e"
    );
    assert_eq!(TL_MLTL_REVISION, "22862189ac4eb515ab84928faec25b2eac47d835");

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
                let (original_request, rewritten_request) =
                    owner_pair(original, rewritten, &values, clock, anchor, &tag);
                let report = check_past_equivalence(
                    &original_request,
                    &rewritten_request,
                    tag,
                    OwnerLimits::default(),
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
            let (original_request, wrong_request) =
                owner_pair(original, wrong, &values, clock, *anchor, &tag);
            let report = check_past_equivalence(
                &original_request,
                &wrong_request,
                tag,
                OwnerLimits::default(),
            );
            assert_eq!(report.status, ConformanceStatus::Mismatch, "{operator}");
            assert_eq!(report.reason, None);
        }
    }

    let original = once(Interval::new(1, 1).unwrap());
    let rewritten = rewrite(&original, "once", RewriteOptions::default(), "source")
        .output
        .unwrap();
    let (original_request, _) = owner_pair(
        &original,
        &rewritten,
        &values,
        FixtureClock::EventPosition,
        0,
        "context-original",
    );
    let (_, different_anchor) = owner_pair(
        &original,
        &rewritten,
        &values,
        FixtureClock::EventPosition,
        1,
        "context-original",
    );
    let refused = check_past_equivalence(
        &original_request,
        &different_anchor,
        "context-mismatch",
        OwnerLimits::default(),
    );
    assert_eq!(refused.status, ConformanceStatus::NonConclusive);
    assert_eq!(refused.reason, Some(PastConformanceReason::ContextMismatch));

    let (original_request, rewritten_request) = owner_pair(
        &original,
        &rewritten,
        &values,
        FixtureClock::EventPosition,
        1,
        "resource-nonvalue",
    );
    let nonvalue = check_past_equivalence(
        &original_request,
        &rewritten_request,
        "resource-nonvalue",
        OwnerLimits {
            max_evaluation_steps: 0,
            ..OwnerLimits::default()
        },
    );
    assert_eq!(nonvalue.status, ConformanceStatus::NonConclusive);
    assert_eq!(
        nonvalue.reason,
        Some(PastConformanceReason::NonBooleanResult)
    );
    assert_eq!(
        nonvalue.original_truth,
        Some(tl_mltl::wire::report::TemporalTruth::Unavailable)
    );
    assert_eq!(
        nonvalue.rewritten_truth,
        Some(tl_mltl::wire::report::TemporalTruth::Unavailable)
    );

    let one_node = constant(true);
    let two_nodes = once(Interval::new(1, 1).unwrap());
    let (short_request, long_request) = owner_pair(
        &one_node,
        &two_nodes,
        &values,
        FixtureClock::EventPosition,
        1,
        "rewritten-owner-refusal",
    );
    let rewritten_refused = check_past_equivalence(
        &short_request,
        &long_request,
        "rewritten-owner-refusal",
        OwnerLimits {
            max_formula_nodes: 1,
            ..OwnerLimits::default()
        },
    );
    assert_eq!(
        rewritten_refused.reason,
        Some(PastConformanceReason::RewrittenEvaluatorError)
    );
    assert!(rewritten_refused.original_result_identity.is_some());
    assert!(rewritten_refused.rewritten_result_identity.is_none());

    let original_refused = check_past_equivalence(
        &long_request,
        &short_request,
        "original-owner-refusal",
        OwnerLimits {
            max_formula_nodes: 1,
            ..OwnerLimits::default()
        },
    );
    assert_eq!(
        original_refused.reason,
        Some(PastConformanceReason::OriginalEvaluatorError)
    );
    assert!(original_refused.original_result_identity.is_none());
    assert!(original_refused.rewritten_result_identity.is_none());
}
