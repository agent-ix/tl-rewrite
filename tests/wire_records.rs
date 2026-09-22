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
    // These are the candidate bytes after the reviewed formula-v2/past-profile
    // dependency advance. The v1 schemas and status meanings did not change;
    // diagnostic spans still do not contribute to formula digests, while the
    // exact compiled syntax/evaluator revisions remain truthful report fields.
    // Updated again for the tl-mltl 0.2.0 / tl-syntax d52d8954 advance (TL-179:
    // dropped the quire-observation dev-dependency): only the compiled
    // TL_SYNTAX_REVISION/TL_MLTL_REVISION report fields changed, so only the
    // digests derived from them move here.
    assert_eq!(
        [
            digest(&rewrite_bytes),
            digest(&replay_bytes),
            digest(&conformance_bytes),
        ],
        [
            "a45ed5125ff8c79e69381eeb2aca38cfb88ae100097a6c6a6113b87fd13a858b",
            "d1ddd8aad30bde4eab954b58a9da36e8fc5c0f4f27d0be1fc2a59d6a88773281",
            "55d446877b30606ea39e1b7cdae1c0c468a0d3aa3a81f13243a4aa87abebaf8b",
        ]
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
