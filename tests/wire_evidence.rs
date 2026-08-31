mod common;

use std::{fs, path::PathBuf, process::Command};

use common::{document, proposition};
use tl_rewrite::{
    catalog, check_equivalence, replay, rewrite, CatalogDocument, ConformanceOptions,
    ConformanceReport, ReplayReport, RewriteOptions, RewriteReport,
};
use tl_syntax::SemanticProfile;

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

// Trace: TC-018, FR-005-AC-2, NFR-002-AC-2, SUITE-001, SUITE-002, SUITE-003
// Trace: SUITE-004, SUITE-005, SUITE-006, SUITE-007
#[test]
fn immutable_evidence_contract_and_schemas_are_complete() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let collector = fs::read_to_string(root.join("scripts/collect_evidence.sh")).unwrap();
    let builder = fs::read_to_string(root.join("scripts/build_evidence_envelope.py")).unwrap();
    let makefile = fs::read_to_string(root.join("Makefile")).unwrap();
    let workflow = fs::read_to_string(root.join(".github/workflows/ci.yml")).unwrap();
    let dry_run = Command::new("make")
        .args(["-n", "ci"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        dry_run.status.success(),
        "make -n ci failed: {}",
        String::from_utf8_lossy(&dry_run.stderr)
    );
    let dry_run = String::from_utf8(dry_run.stdout).unwrap();
    for required in [
        "refusing to overwrite retained evidence",
        "refusing to collect evidence from a modified or untracked source tree",
        "PGM01_SCHEMA",
        "PGM01_PYTHON",
        "PGM01_VALIDATOR",
        "make ci",
        "make spec",
        "quire coverage --scope . --strict",
        "sealed-pgm01-schema",
        "sealed-pgm01-validator",
        "finalize_collection.py",
    ] {
        assert!(collector.contains(required), "collector omits {required}");
    }
    for required in [
        "TL_SYNTAX_REVISION",
        "TL_MLTL_REVISION",
        "WEST_REVISION",
        "PGM01_POLICY_REVISION",
        "componentClass",
        "reviewState",
    ] {
        assert!(builder.contains(required), "builder omits {required}");
    }
    for target in [
        "ci:",
        "spec:",
        "check-corpus:",
        "evidence-tool:",
        "verify-evidence:",
    ] {
        assert!(makefile.contains(target));
    }
    for command in [
        "cargo deny check licenses",
        "cargo deny check sources",
        "python3 scripts/test_evidence_tool.py",
        "bash scripts/verify_evidence.sh",
        "quire validate",
        "quire coverage",
        "cargo doc",
    ] {
        assert!(dry_run.contains(command), "complete gate omits {command}");
    }
    assert!(workflow.contains("workflow_dispatch:"));
    assert!(!workflow.contains("pull_request:"));
    assert!(!workflow.contains("push:"));
    for script in [
        "scripts/finalize_collection.py",
        "scripts/test_evidence_tool.py",
        "scripts/verify_evidence.sh",
    ] {
        assert!(root.join(script).is_file(), "missing {script}");
    }
    for schema in [
        "schemas/tl-rewrite-evidence-input-v1.schema.json",
        "schemas/tl-rewrite-evidence-manifest-v1.schema.json",
    ] {
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join(schema)).unwrap()).unwrap();
        assert_eq!(value["$schema"], "http://json-schema.org/draft-07/schema#");
        assert_eq!(value["additionalProperties"], false);
    }
}

#[test]
fn evidence_producer_rejects_false_success_classifications() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new("python3")
        .arg("scripts/test_evidence_tool.py")
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "evidence behavior test failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// Trace: TC-019, FR-005-AC-3, StR-001-VC-1, NFR-002-AC-2
#[test]
fn human_authority_and_qualification_boundaries_remain_open() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let combined = [
        "README.md",
        "evidence/README.md",
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
