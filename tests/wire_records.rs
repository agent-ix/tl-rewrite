mod common;

use common::{document, proposition};
use sha2::{Digest, Sha256};
use tl_rewrite::{
    catalog, check_equivalence, replay, rewrite, CatalogDocument, ConformanceOptions,
    ConformanceReport, ReplayReport, RewriteOptions, RewriteReport,
};
use tl_syntax::{Node, SemanticProfile, SourceSpan};

fn add_unknown(mut value: serde_json::Value) -> serde_json::Value {
    value
        .as_object_mut()
        .unwrap()
        .insert("unknown".to_owned(), serde_json::Value::Bool(true));
    value
}

// Trace: TC-017, FR-005-AC-1, NFR-001-AC-1
#[test]
fn versioned_records_round_trip_and_reject_unknown_fields() {
    let input = document(SemanticProfile::ClosedTraceV1, vec![proposition(0)]);
    let rewrite_report = rewrite(&input, "wire", RewriteOptions::default(), "source");
    let replay_report = replay(&input, &rewrite_report);
    let conformance = check_equivalence(
        &input,
        rewrite_report.output.as_ref().unwrap(),
        "wire",
        ConformanceOptions::default(),
    );
    let catalog = catalog();

    let catalog_value = serde_json::to_value(&catalog).unwrap();
    assert_eq!(
        serde_json::from_value::<CatalogDocument>(catalog_value.clone()).unwrap(),
        catalog
    );
    assert!(serde_json::from_value::<CatalogDocument>(add_unknown(catalog_value)).is_err());

    let rewrite_value = serde_json::to_value(&rewrite_report).unwrap();
    assert_eq!(
        serde_json::from_value::<RewriteReport>(rewrite_value.clone()).unwrap(),
        rewrite_report
    );
    assert!(serde_json::from_value::<RewriteReport>(add_unknown(rewrite_value)).is_err());

    let replay_value = serde_json::to_value(&replay_report).unwrap();
    assert_eq!(
        serde_json::from_value::<ReplayReport>(replay_value.clone()).unwrap(),
        replay_report
    );
    assert!(serde_json::from_value::<ReplayReport>(add_unknown(replay_value)).is_err());

    let conformance_value = serde_json::to_value(&conformance).unwrap();
    assert_eq!(
        serde_json::from_value::<ConformanceReport>(conformance_value.clone()).unwrap(),
        conformance
    );
    assert!(serde_json::from_value::<ConformanceReport>(add_unknown(conformance_value)).is_err());
}

// Trace: TC-035, TC-046, TC-053, FR-002-AC-4, FR-007-AC-5, FR-009-AC-1, FR-009-AC-6, FR-010-AC-3
#[test]
fn context_free_report_families_keep_their_v01_semantic_identity_bytes() {
    let input = document(
        SemanticProfile::ClosedTraceV1,
        vec![Node::with_span(
            proposition(0).kind,
            SourceSpan::new(4, 7).unwrap(),
        )],
    );
    let rewrite_report = rewrite(&input, "v1-snapshot", RewriteOptions::default(), "source");
    let replay_report = replay(&input, &rewrite_report);
    let conformance = check_equivalence(
        &input,
        rewrite_report.output.as_ref().unwrap(),
        "v1-snapshot",
        ConformanceOptions::default(),
    );
    let digest = |value: &[u8]| format!("{:x}", Sha256::digest(value));
    let rewrite_bytes = serde_json::to_vec(&rewrite_report).unwrap();
    let replay_bytes = serde_json::to_vec(&replay_report).unwrap();
    let conformance_bytes = serde_json::to_vec(&conformance).unwrap();
    assert_eq!(
        [
            digest(&rewrite_bytes),
            digest(&replay_bytes),
            digest(&conformance_bytes),
        ],
        [
            "46806f2ecf30203bc8e9c755e95361e701f2e027e6faaa5306cd1edc1cd799ee",
            "da23695a8004391635e0d2b7fde6c5ed57283ec63e652fd27a9aca8e1933867a",
            "d7b1f4541fc6b5f6779475ed36ed41398d72064b286ff5065584fbdb48e9d523",
        ]
    );
    assert_eq!(rewrite_report.schema_version, "tl-rewrite.report/v1");
    assert_eq!(replay_report.schema_version, "tl-rewrite.replay/v1");
    assert_eq!(conformance.schema_version, "tl-rewrite.conformance/v1");
}
