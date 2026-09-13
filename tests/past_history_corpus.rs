use std::{collections::BTreeMap, fs, path::Path};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use tl_rewrite::{replay, rewrite, ReplayStatus, RewriteOptions, RewriteStatus};
use tl_syntax::{FormulaDocument, NodeKind};

const DIRECTORY: &str = "corpus/past-history";
const MANIFEST_SHA256: &str = "59b86e7c888bf850cdd4e49cf86b01ffb64a99887d7fd051dca6a7d9f0a56393";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    corpus: String,
    revision: u64,
    role: String,
    formula_schema: String,
    operator_profile: String,
    semantic_profile: String,
    history_schema: String,
    dialect: String,
    implementation_revisions: Revisions,
    files: Vec<Pin>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revisions {
    tl_syntax: String,
    tl_parse: String,
    tl_mltl: String,
    tl_rewrite: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pin {
    path: String,
    sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cases {
    corpus: String,
    formula_schema: String,
    operator_profile: String,
    semantic_profile: String,
    dialect: String,
    formulas: Vec<FormulaCase>,
    histories: Vec<serde_json::Value>,
    evaluations: Vec<serde_json::Value>,
    rewrites: Vec<RewriteCase>,
    refusals: Vec<serde_json::Value>,
    target_dispositions: Vec<serde_json::Value>,
    mutation_axes: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FormulaCase {
    id: String,
    source: String,
    required_history: u64,
    document: FormulaDocument,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RewriteCase {
    id: String,
    input: String,
    expected_rule: Option<String>,
    expected_status: String,
    expected_source: String,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn load() -> (Manifest, Cases) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(DIRECTORY);
    let manifest_bytes = fs::read(root.join("manifest.json")).unwrap();
    assert_eq!(digest(&manifest_bytes), MANIFEST_SHA256);
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes).unwrap();
    let pins: BTreeMap<_, _> = manifest
        .files
        .iter()
        .map(|pin| (&pin.path, &pin.sha256))
        .collect();
    assert_eq!(pins.len(), 3);
    for pin in &manifest.files {
        assert_eq!(
            digest(&fs::read(root.join(&pin.path)).unwrap()),
            pin.sha256,
            "{}",
            pin.path
        );
    }
    let cases = serde_json::from_slice(&fs::read(root.join("cases.json")).unwrap()).unwrap();
    (manifest, cases)
}

// Trace: TC-056, FR-009-AC-2, FR-009-AC-4, FR-013-AC-3
#[test]
fn exact_shared_corpus_replays_every_reviewed_rewrite_and_identity_case() {
    let (manifest, cases) = load();
    assert_eq!(manifest.corpus, "tl-syntax.past-history-corpus/v1");
    assert_eq!(manifest.revision, 1);
    assert_eq!(manifest.role, "evidence-input");
    assert_eq!(manifest.formula_schema, "tl-syntax.formula/v2");
    assert_eq!(manifest.operator_profile, "tl-syntax.past-operators/v1");
    assert_eq!(manifest.semantic_profile, "mltl.origin-complete-history/v1");
    assert_eq!(manifest.history_schema, "tl-mltl.position-history/v1");
    assert_eq!(manifest.dialect, "tl-parse.clean-ascii/v3");
    assert_eq!(
        manifest.implementation_revisions.tl_syntax,
        "e70f2379a752117c79603bc399a86c26feed7716"
    );
    assert_eq!(
        manifest.implementation_revisions.tl_parse,
        "f82b0c724675c0f774415aa696c360959da30481"
    );
    assert_eq!(
        manifest.implementation_revisions.tl_mltl,
        "b346cd0902794633e862f644a5575fc9776c34fb"
    );
    assert_eq!(
        manifest.implementation_revisions.tl_rewrite,
        "22b9cadcb1692cec8d3a97768f4f3b38fc654a5e"
    );
    assert_eq!(cases.corpus, manifest.corpus);
    assert_eq!(cases.formula_schema, manifest.formula_schema);
    assert_eq!(cases.operator_profile, manifest.operator_profile);
    assert_eq!(cases.semantic_profile, manifest.semantic_profile);
    assert_eq!(cases.dialect, manifest.dialect);
    assert_eq!(cases.histories.len(), 3);
    assert_eq!(cases.evaluations.len(), 9);
    assert_eq!(cases.refusals.len(), 12);
    assert_eq!(cases.target_dispositions.len(), 4);
    assert_eq!(cases.mutation_axes.len(), 10);

    let formulas: BTreeMap<_, _> = cases
        .formulas
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect();
    assert_eq!(formulas.len(), cases.formulas.len());
    for formula in &cases.formulas {
        assert!(!formula.source.is_empty());
        assert!(formula.required_history <= u64::from(u32::MAX));
        formula.document.validate().unwrap();
    }

    for case in &cases.rewrites {
        assert!(!case.id.is_empty());
        let input = &formulas[case.input.as_str()].document;
        let report = rewrite(
            input,
            &case.id,
            RewriteOptions::default(),
            "shared-past-corpus/v1",
        );
        match case.expected_status.as_str() {
            "rewritten" => {
                assert_eq!(report.status, RewriteStatus::Normalized, "{}", case.id);
                assert_eq!(report.steps.len(), 1, "{}", case.id);
                assert_eq!(
                    report.steps[0].rule_id,
                    case.expected_rule.as_deref().unwrap(),
                    "{}",
                    case.id
                );
            }
            "unchanged" => {
                assert_eq!(report.status, RewriteStatus::Unchanged, "{}", case.id);
                assert!(report.steps.is_empty(), "{}", case.id);
                assert!(case.expected_rule.is_none());
            }
            unknown => panic!("unknown corpus rewrite status {unknown}"),
        }
        let output = report.output.as_ref().unwrap();
        assert_eq!(output.schema_version(), input.schema_version());
        assert_eq!(output.semantic_profile(), input.semantic_profile());
        match case.expected_source.as_str() {
            "Yp0" => assert!(matches!(
                output.nodes()[output.root().0 as usize].kind,
                NodeKind::StrongPrevious { .. }
            )),
            "p0T[1,2]p1" => assert!(
                matches!(output.nodes()[output.root().0 as usize].kind, NodeKind::Triggered { interval, .. } if interval.start() == 1 && interval.end() == 2)
            ),
            "p0S[1,2]p1" => assert_eq!(
                serde_json::to_value(output.semantic_view()).unwrap(),
                serde_json::to_value(input.semantic_view()).unwrap()
            ),
            unknown => panic!("unreplayed expected source {unknown}"),
        }
        assert_eq!(
            replay(input, &report).status,
            ReplayStatus::Verified,
            "{}",
            case.id
        );
    }
}

// Trace: TC-056, FR-009-AC-4, FR-013-AC-3
#[test]
fn corpus_digest_and_rewrite_expectation_mutations_are_detected() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(DIRECTORY);
    let mut manifest = fs::read(root.join("manifest.json")).unwrap();
    manifest[0] ^= 1;
    assert_ne!(digest(&manifest), MANIFEST_SHA256);

    let (_, cases) = load();
    let input = &cases
        .formulas
        .iter()
        .find(|item| item.id == "once-one-rewrite")
        .unwrap()
        .document;
    let mut report = rewrite(
        input,
        "mutation",
        RewriteOptions::default(),
        "shared-past-corpus/v1",
    );
    report.steps[0].rule_id = "mutated.rule".to_owned();
    assert_eq!(replay(input, &report).status, ReplayStatus::Mismatch);
}
