mod common;

use common::{document, proposition};
use tl_rewrite::{
    check_equivalence_with_context, replay_with_context, rewrite_with_context, BindingLocus,
    ConformanceOptions, ConformanceReason, ConformanceStatus, ReplayStatus, RewriteOptions,
    RewriteStatus,
};
use tl_syntax::{
    IntegerSignalDomain, Node, NodeId, NodeKind, OwnedSignalDeclaration, PropositionBinding,
    PropositionId, RequirementContextDocument, SemanticProfile, SignalCatalogDocument,
    SignalDomain, SignalId, SourceSpan,
};

fn catalog_with_name(binding: u32, name: &str) -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        vec![OwnedSignalDeclaration::new(
            SignalId(11),
            name.to_owned(),
            SignalDomain::Boolean,
        )],
        vec![PropositionBinding::new(
            PropositionId(binding),
            SignalId(11),
        )],
    )
    .unwrap()
}

fn catalog(binding: u32) -> SignalCatalogDocument {
    catalog_with_name(binding, "request_ready")
}

fn catalog_with_unused(
    binding: u32,
    unused_name: &str,
    unused_domain: SignalDomain,
) -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        vec![
            OwnedSignalDeclaration::new(
                SignalId(11),
                "request_ready".to_owned(),
                SignalDomain::Boolean,
            ),
            OwnedSignalDeclaration::new(SignalId(12), unused_name.to_owned(), unused_domain),
        ],
        vec![PropositionBinding::new(
            PropositionId(binding),
            SignalId(11),
        )],
    )
    .unwrap()
}

fn context_fields(
    requirement_id: &str,
    revision: &str,
    clause_id: &str,
    anchor: &str,
    start: u32,
    end: u32,
) -> RequirementContextDocument {
    RequirementContextDocument::new(
        requirement_id.to_owned(),
        revision.to_owned(),
        clause_id.to_owned(),
        anchor.to_owned(),
        SourceSpan::new(start, end).unwrap(),
    )
    .unwrap()
}

fn context_with(anchor: &str) -> RequirementContextDocument {
    context_fields("agent-ix/demo/FR-007", "1", "AC-1", anchor, 10, 24)
}

fn context() -> RequirementContextDocument {
    context_with("demo.context")
}

// Trace: TC-031, TC-033, FR-007-AC-1, FR-007-AC-3
#[test]
fn contextual_rewrite_carries_exact_context_and_refuses_missing_input_binding() {
    let input = document(SemanticProfile::ClosedTraceV1, vec![proposition(7)]);
    let supplied_context = context();
    let report = rewrite_with_context(
        &input,
        "contextual-positive",
        RewriteOptions::default(),
        "source",
        &catalog(7),
        Some(supplied_context.clone()),
    );
    assert_eq!(report.schema_version, "tl-rewrite.report/v2");
    assert_eq!(report.requirement_context, Some(Some(supplied_context)));
    assert!(report.signal_catalog_sha256.is_some());
    assert!(report.binding_failure.is_none());
    assert!(report.output.is_some());

    let refused = rewrite_with_context(
        &input,
        "contextual-refusal",
        RewriteOptions::default(),
        "source",
        &catalog(8),
        None,
    );
    assert_eq!(refused.status, RewriteStatus::UnresolvedBinding);
    assert_eq!(
        refused.binding_failure.unwrap(),
        tl_rewrite::BindingFailure {
            locus: BindingLocus::Input,
            proposition_id: 7,
        }
    );
    assert!(refused.output.is_none());
    assert_eq!(refused.requirement_context, Some(None));
}

// Trace: TC-032, FR-007-AC-2
#[test]
fn contextual_replay_binds_the_resupplied_catalog_and_context() {
    let input = document(SemanticProfile::ClosedTraceV1, vec![proposition(7)]);
    let supplied_context = context();
    let supplied_catalog = catalog(7);
    let expected = rewrite_with_context(
        &input,
        "contextual-replay",
        RewriteOptions::default(),
        "source",
        &supplied_catalog,
        Some(supplied_context.clone()),
    );
    let verified =
        replay_with_context(&input, &expected, &supplied_catalog, Some(supplied_context));
    assert_eq!(verified.schema_version, "tl-rewrite.replay/v2");
    assert_eq!(verified.status, ReplayStatus::Verified);
    assert_eq!(
        serde_json::from_value::<tl_rewrite::ReplayReport>(
            serde_json::to_value(&verified).unwrap()
        )
        .unwrap(),
        verified
    );
    let mut missing_context = serde_json::to_value(&verified).unwrap();
    missing_context
        .as_object_mut()
        .unwrap()
        .remove("requirementContext");
    assert!(serde_json::from_value::<tl_rewrite::ReplayReport>(missing_context).is_err());

    let changed_context = replay_with_context(
        &input,
        &expected,
        &supplied_catalog,
        Some(context_with("demo.changed")),
    );
    assert_eq!(changed_context.status, ReplayStatus::Mismatch);

    let changed_catalog = replay_with_context(
        &input,
        &expected,
        &catalog_with_name(7, "request_ready_changed"),
        Some(context()),
    );
    assert_eq!(changed_catalog.status, ReplayStatus::Mismatch);
}

// Trace: TC-032, FR-007-AC-2
#[test]
fn contextual_replay_rejects_each_independent_request_substitution() {
    let input = document(
        SemanticProfile::ClosedTraceV1,
        vec![
            proposition(7),
            Node::new(NodeKind::Or {
                left: NodeId(0),
                right: NodeId(0),
            }),
        ],
    );
    let supplied_catalog = catalog_with_unused(7, "unused", SignalDomain::Boolean);
    let supplied_context = context();
    let expected = rewrite_with_context(
        &input,
        "contextual-mutation-table",
        RewriteOptions::default(),
        "source",
        &supplied_catalog,
        Some(supplied_context.clone()),
    );
    assert!(!expected.steps.is_empty());
    assert_eq!(
        replay_with_context(
            &input,
            &expected,
            &supplied_catalog,
            Some(supplied_context.clone()),
        )
        .status,
        ReplayStatus::Verified
    );

    // Independent catalog declaration, domain, name, and binding changes.
    assert_eq!(
        replay_with_context(
            &input,
            &expected,
            &catalog(7),
            Some(supplied_context.clone()),
        )
        .status,
        ReplayStatus::Mismatch
    );
    assert_eq!(
        replay_with_context(
            &input,
            &expected,
            &catalog_with_unused(
                7,
                "unused",
                SignalDomain::Integer(IntegerSignalDomain::new(-1, 1).unwrap()),
            ),
            Some(supplied_context.clone()),
        )
        .status,
        ReplayStatus::Mismatch
    );
    assert_eq!(
        replay_with_context(
            &input,
            &expected,
            &catalog_with_unused(7, "unused_renamed", SignalDomain::Boolean),
            Some(supplied_context.clone()),
        )
        .status,
        ReplayStatus::Mismatch
    );
    assert_eq!(
        replay_with_context(
            &input,
            &expected,
            &catalog_with_unused(8, "unused", SignalDomain::Boolean),
            Some(supplied_context.clone()),
        )
        .status,
        ReplayStatus::Mismatch
    );

    // Each context value and the explicit presence marker participates.
    for changed_context in [
        context_fields(
            "agent-ix/demo/FR-007-changed",
            "1",
            "AC-1",
            "demo.context",
            10,
            24,
        ),
        context_fields("agent-ix/demo/FR-007", "2", "AC-1", "demo.context", 10, 24),
        context_fields("agent-ix/demo/FR-007", "1", "AC-2", "demo.context", 10, 24),
        context_fields("agent-ix/demo/FR-007", "1", "AC-1", "demo.changed", 10, 24),
        context_fields("agent-ix/demo/FR-007", "1", "AC-1", "demo.context", 11, 24),
    ] {
        assert_eq!(
            replay_with_context(&input, &expected, &supplied_catalog, Some(changed_context)).status,
            ReplayStatus::Mismatch
        );
    }
    assert_eq!(
        replay_with_context(&input, &expected, &supplied_catalog, None).status,
        ReplayStatus::Mismatch
    );

    let changed_input = document(SemanticProfile::ClosedTraceV1, vec![proposition(7)]);
    assert_eq!(
        replay_with_context(
            &changed_input,
            &expected,
            &supplied_catalog,
            Some(supplied_context),
        )
        .status,
        ReplayStatus::Mismatch
    );

    let mut changed = expected.clone();
    changed.options.budgets.max_work_units += 1;
    assert_eq!(
        replay_with_context(&input, &changed, &supplied_catalog, Some(context())).status,
        ReplayStatus::Mismatch
    );
    changed = expected.clone();
    changed.catalog_sha256 = "0".repeat(64);
    assert_eq!(
        replay_with_context(&input, &changed, &supplied_catalog, Some(context())).status,
        ReplayStatus::Mismatch
    );
    changed = expected;
    changed.steps[0].intermediate_sha256 = "f".repeat(64);
    assert_eq!(
        replay_with_context(&input, &changed, &supplied_catalog, Some(context())).status,
        ReplayStatus::Mismatch
    );
}

// Trace: TC-034, FR-007-AC-4
#[test]
fn contextual_equivalence_refuses_each_formula_before_enumeration() {
    let original = document(SemanticProfile::ClosedTraceV1, vec![proposition(7)]);
    let equivalent = check_equivalence_with_context(
        &original,
        &original,
        "contextual-equivalence",
        ConformanceOptions::default(),
        &catalog(7),
        Some(context()),
    );
    assert_eq!(equivalent.schema_version, "tl-rewrite.conformance/v2");
    assert_eq!(equivalent.status, ConformanceStatus::Equivalent);
    assert!(equivalent.request_sha256.is_some());
    assert_eq!(
        serde_json::from_value::<tl_rewrite::ConformanceReport>(
            serde_json::to_value(&equivalent).unwrap()
        )
        .unwrap(),
        equivalent
    );
    let mut missing_context = serde_json::to_value(&equivalent).unwrap();
    missing_context
        .as_object_mut()
        .unwrap()
        .remove("requirementContext");
    assert!(serde_json::from_value::<tl_rewrite::ConformanceReport>(missing_context).is_err());

    let v1 = tl_rewrite::check_equivalence(
        &original,
        &original,
        "context-free-conformance",
        ConformanceOptions::default(),
    );
    let mut smuggled = serde_json::to_value(v1).unwrap();
    smuggled.as_object_mut().unwrap().insert(
        "requestSha256".to_owned(),
        serde_json::Value::String("forged".to_owned()),
    );
    assert!(serde_json::from_value::<tl_rewrite::ConformanceReport>(smuggled).is_err());

    let missing_original = check_equivalence_with_context(
        &original,
        &original,
        "missing-original",
        ConformanceOptions::default(),
        &catalog(8),
        None,
    );
    assert_eq!(
        missing_original.reason,
        Some(ConformanceReason::OriginalBinding)
    );
    assert_eq!(missing_original.traces_checked, 0);

    let rewritten = document(SemanticProfile::ClosedTraceV1, vec![proposition(8)]);
    let missing_rewritten = check_equivalence_with_context(
        &original,
        &rewritten,
        "missing-rewritten",
        ConformanceOptions::default(),
        &catalog(7),
        None,
    );
    assert_eq!(
        missing_rewritten.reason,
        Some(ConformanceReason::RewrittenBinding)
    );
    assert_eq!(missing_rewritten.traces_checked, 0);
}

// Trace: TC-035, FR-007-AC-5
#[test]
fn contextual_rewrite_wire_requires_v2_context_and_rejects_v1_smuggling() {
    let input = document(SemanticProfile::ClosedTraceV1, vec![proposition(7)]);
    let report = rewrite_with_context(
        &input,
        "contextual-wire",
        RewriteOptions::default(),
        "source",
        &catalog(7),
        None,
    );
    let value = serde_json::to_value(&report).unwrap();
    assert_eq!(
        serde_json::from_value::<tl_rewrite::RewriteReport>(value.clone()).unwrap(),
        report
    );
    let mut missing_context = value.clone();
    missing_context
        .as_object_mut()
        .unwrap()
        .remove("requirementContext");
    assert!(serde_json::from_value::<tl_rewrite::RewriteReport>(missing_context).is_err());

    let v1 = tl_rewrite::rewrite(&input, "v1", RewriteOptions::default(), "source");
    let mut smuggled = serde_json::to_value(v1).unwrap();
    smuggled
        .as_object_mut()
        .unwrap()
        .insert("requirementContext".to_owned(), serde_json::Value::Null);
    assert!(serde_json::from_value::<tl_rewrite::RewriteReport>(smuggled).is_err());
}

// Trace: TC-035, FR-007-AC-5
#[test]
fn every_contextual_native_wire_family_is_closed_and_versioned() {
    let input = document(SemanticProfile::ClosedTraceV1, vec![proposition(7)]);
    let supplied_catalog = catalog(7);
    let report = rewrite_with_context(
        &input,
        "closed-v2-wire",
        RewriteOptions::default(),
        "source",
        &supplied_catalog,
        Some(context()),
    );
    let replay = replay_with_context(&input, &report, &supplied_catalog, Some(context()));
    let conformance = check_equivalence_with_context(
        &input,
        report.output.as_ref().unwrap(),
        "closed-v2-wire",
        ConformanceOptions::default(),
        &supplied_catalog,
        Some(context()),
    );

    let rewrite_value = serde_json::to_value(&report).unwrap();
    let replay_value = serde_json::to_value(&replay).unwrap();
    let conformance_value = serde_json::to_value(&conformance).unwrap();
    for value in [
        rewrite_value.clone(),
        replay_value.clone(),
        conformance_value.clone(),
    ] {
        let mut missing_identity = value.clone();
        missing_identity
            .as_object_mut()
            .unwrap()
            .remove("signalCatalogSha256");
        let mut unknown = value.clone();
        unknown
            .as_object_mut()
            .unwrap()
            .insert("unknown".to_owned(), serde_json::Value::Bool(true));
        let mut unsupported = value;
        unsupported.as_object_mut().unwrap().insert(
            "schemaVersion".to_owned(),
            serde_json::Value::String("tl-rewrite.contextual/unsupported".to_owned()),
        );

        // Each concrete assertion below keeps the test bound to the public
        // report family rather than hiding a deserialize target in a helper.
        if missing_identity.get("expectedReportSha256").is_some() {
            assert!(serde_json::from_value::<tl_rewrite::ReplayReport>(missing_identity).is_err());
            assert!(serde_json::from_value::<tl_rewrite::ReplayReport>(unknown).is_err());
            assert!(serde_json::from_value::<tl_rewrite::ReplayReport>(unsupported).is_err());
        } else if missing_identity.get("comparisonId").is_some() {
            assert!(
                serde_json::from_value::<tl_rewrite::ConformanceReport>(missing_identity).is_err()
            );
            assert!(serde_json::from_value::<tl_rewrite::ConformanceReport>(unknown).is_err());
            assert!(serde_json::from_value::<tl_rewrite::ConformanceReport>(unsupported).is_err());
        } else {
            assert!(serde_json::from_value::<tl_rewrite::RewriteReport>(missing_identity).is_err());
            assert!(serde_json::from_value::<tl_rewrite::RewriteReport>(unknown).is_err());
            assert!(serde_json::from_value::<tl_rewrite::RewriteReport>(unsupported).is_err());
        }
    }

    let mut invalid_context = rewrite_value;
    invalid_context.as_object_mut().unwrap().insert(
        "requirementContext".to_owned(),
        serde_json::json!({ "not": "a requirement context" }),
    );
    assert!(serde_json::from_value::<tl_rewrite::RewriteReport>(invalid_context).is_err());

    let v1_replay = tl_rewrite::replay(
        &input,
        &tl_rewrite::rewrite(&input, "v1-replay", RewriteOptions::default(), "source"),
    );
    let mut smuggled_replay = serde_json::to_value(v1_replay).unwrap();
    smuggled_replay.as_object_mut().unwrap().insert(
        "signalCatalogSha256".to_owned(),
        serde_json::Value::String("forged".to_owned()),
    );
    assert!(serde_json::from_value::<tl_rewrite::ReplayReport>(smuggled_replay).is_err());
}
