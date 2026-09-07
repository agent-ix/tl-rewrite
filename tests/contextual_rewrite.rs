mod common;

use common::{document, proposition};
use tl_rewrite::{
    check_equivalence_with_context, replay_with_context, rewrite_with_context, BindingLocus,
    ConformanceOptions, ConformanceReason, ConformanceStatus, ReplayStatus, RewriteOptions,
    RewriteStatus,
};
use tl_syntax::{
    OwnedSignalDeclaration, PropositionBinding, PropositionId, RequirementContextDocument,
    SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId, SourceSpan,
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

fn context_with(anchor: &str) -> RequirementContextDocument {
    RequirementContextDocument::new(
        "agent-ix/demo/FR-007".to_owned(),
        "1".to_owned(),
        "AC-1".to_owned(),
        anchor.to_owned(),
        SourceSpan::new(10, 24).unwrap(),
    )
    .unwrap()
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
