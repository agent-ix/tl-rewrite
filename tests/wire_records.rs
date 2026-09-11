mod common;

use std::{fs, path::PathBuf};

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

// Trace: TC-035, FR-002-AC-4, FR-007-AC-5
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
    // These are the frozen v0.1 bytes after the one pre-release semantic-
    // identity correction required by FR-002-AC-4. The v1 schemas did not
    // change; diagnostic spans no longer contribute to formula digests.
    assert_eq!(
        digest(&rewrite_bytes),
        "1645c0ec6d22a866146ee9ee240fcb22364beea659f1fb49dfd3e45cfa92f784"
    );
    assert_eq!(
        digest(&replay_bytes),
        "9c66888b5d9e5413a6ee6cdf5ff491666cd471991d999e0edd65fc2c898b198c"
    );
    assert_eq!(
        digest(&conformance_bytes),
        "20de1baa30fcde19d1c547f21caa502c2cac957f43c191b523e40f24b585a080"
    );
    assert_eq!(rewrite_report.schema_version, "tl-rewrite.report/v1");
    assert_eq!(replay_report.schema_version, "tl-rewrite.replay/v1");
    assert_eq!(conformance.schema_version, "tl-rewrite.conformance/v1");
}

// Trace: TC-019, FR-005-AC-3, StR-001-VC-1
#[test]
fn human_authority_and_qualification_boundaries_remain_open() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Four documents, not five. `evidence/README.md` was inspected here until
    // issue #13 deleted it with the archive it described. It was the only source
    // of `pending` — it said the retained records "inform a pending human
    // source-release decision". The property is still true and is not dropped:
    // the statement moved to AA-001's Human Decision section, which is the
    // document that owns the claim, so this inspection asserts the same six
    // properties over a smaller set rather than five properties over four files.
    let combined = [
        "README.md",
        "spec/assurance/AP-001.md",
        "spec/assurance/AA-001.md",
        "docs/DER-001-rule-derivations.md",
    ]
    .iter()
    .map(|path| fs::read_to_string(root.join(path)).unwrap())
    .collect::<Vec<_>>()
    .join("\n");
    for required in [
        "@kreneskyp",
        "human",
        "pending",
        "does not prove arbitrary",
        "qualif",
        "source-release",
    ] {
        assert!(combined.contains(required), "boundary omits {required}");
    }
}
