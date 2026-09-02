//! Replay every catalog rule through the real engine and oracle (FR-006-AC-2).
//!
//! This is a producer. It runs the actual rewrite engine and the actual pinned
//! `tl-mltl` evaluator over the checked rule corpus and writes one JSON object
//! per catalog rule to stdout. It computes no aggregate verdict, retains
//! nothing, and knows nothing about Quoin, Quire or attestations — the assurance
//! chain reads the rows this writes and reports what they say.
//!
//! Two outcome words carry weight here and are deliberately neither `pass` nor
//! `fail`.
//!
//! `unsupported` is what an *excluded* catalog rule produces. The two
//! `west.nested-*` rules are retained for review and are not executable in v1;
//! reporting them as `pass` would make a catalog that quietly enabled them
//! indistinguishable from one that still refuses, and reporting them as `fail`
//! would report a permanently failing proof for a deliberate and documented
//! exclusion.
//!
//! `not-computed` is what a comparison that reached no verdict produces. Bounded
//! enumeration that stopped at a declared ceiling has not agreed; it has
//! declined to answer, and that is a third fact.
//!
//! `fail` is reserved for genuine disagreement: a rule that did not fire, a rule
//! that fired with a revision the catalog does not name, a fixture whose bytes
//! are not the bytes the corpus declares, or a rewrite the oracle found a
//! counterexample to.

use std::{collections::BTreeSet, fs, path::PathBuf, process::ExitCode};

use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tl_rewrite::{
    catalog, check_equivalence, rewrite, ConformanceOptions, ConformanceReport, ConformanceStatus,
    RewriteOptions, RewriteStatus, RuleDisposition,
};
use tl_syntax::FormulaDocument;

const PROTOCOL: &str = "tl-rewrite.rule-conformance/v1";
const SCHEMA: &str = "tl-rewrite.rule-corpus/v1";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    schema_version: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    id: String,
    disposition: String,
    expected_outcome: String,
    #[serde(default)]
    expected_rule: Option<String>,
    #[serde(default)]
    formula_sha256: Option<String>,
    #[serde(default)]
    document: Option<Value>,
    #[serde(default)]
    exclusion_reason: Option<String>,
}

struct Row {
    symbol: String,
    outcome: &'static str,
    domain_outcome: String,
    detail: String,
    extra: Value,
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn failure(symbol: &str, domain: &str, detail: String) -> Row {
    Row {
        symbol: symbol.to_owned(),
        outcome: "fail",
        domain_outcome: domain.to_owned(),
        detail,
        extra: json!({}),
    }
}

fn non_conclusive_label(report: &ConformanceReport) -> String {
    let reason = report
        .reason
        .map(|value| {
            serde_json::to_value(value)
                .ok()
                .and_then(|item| item.as_str().map(str::to_owned))
                .unwrap_or_else(|| "unknown".to_owned())
        })
        .unwrap_or_else(|| "unstated".to_owned());
    format!("non_conclusive:{reason}")
}

/// Classify one catalog rule by executing it, never by consulting a stored answer.
fn evaluate(case: &Case, rule: &tl_rewrite::RuleDefinition) -> Row {
    let symbol = format!("rule:{}", case.id);

    if case.disposition == "excluded" {
        // The obligation for an excluded rule is that it stays excluded: the
        // catalog must still say so, must still carry the primary-source
        // provenance that justifies retaining it, and must still name a reason.
        if rule.disposition != RuleDisposition::Excluded {
            return failure(
                &symbol,
                "unsupported",
                format!(
                    "{} is declared excluded by the corpus but the catalog now enables it",
                    case.id
                ),
            );
        }
        let Some(reason) = rule.exclusion_reason.as_ref() else {
            return failure(
                &symbol,
                "unsupported",
                format!("{} is excluded but names no exclusion reason", case.id),
            );
        };
        if case.exclusion_reason.as_deref() != Some(reason.as_str()) {
            return failure(
                &symbol,
                "unsupported",
                format!(
                    "{} exclusion reason drifted from the corpus declaration",
                    case.id
                ),
            );
        }
        return Row {
            symbol,
            outcome: "unsupported",
            domain_outcome: "unsupported".to_owned(),
            detail: format!(
                "{} is retained for review and is not executable in v1",
                case.id
            ),
            extra: json!({ "exclusionReason": reason }),
        };
    }

    let Some(declared) = case.document.as_ref() else {
        return failure(
            &symbol,
            "malformed_input",
            format!("{} declares no input document", case.id),
        );
    };
    let Some(expected_digest) = case.formula_sha256.as_ref() else {
        return failure(
            &symbol,
            "malformed_input",
            format!("{} declares no formulaSha256", case.id),
        );
    };

    let input: FormulaDocument = match serde_json::from_value(declared.clone()) {
        Ok(value) => value,
        Err(error) => {
            // A corpus document that will not decode is malformed. tl-rewrite
            // consumes documents tl-syntax has already validated, so this is a
            // defect on the wire rather than an input class this crate exists
            // to tolerate.
            return Row {
                symbol,
                outcome: "malformed",
                domain_outcome: "malformed_input".to_owned(),
                detail: format!("{} did not decode: {error}", case.id),
                extra: json!({}),
            };
        }
    };

    // The fixture bytes are bound to the corpus. Without this the corpus is a
    // description of a document nobody checks, and a silently edited fixture
    // would still report a clean rule.
    let observed_digest = match serde_json::to_vec(&input) {
        Ok(bytes) => digest(&bytes),
        Err(error) => {
            return failure(
                &symbol,
                "malformed_input",
                format!("{} did not re-serialize: {error}", case.id),
            )
        }
    };
    if &observed_digest != expected_digest {
        return failure(
            &symbol,
            "tampered",
            format!(
                "{} document digest is {observed_digest}, the corpus declares {expected_digest}",
                case.id
            ),
        );
    }

    let report = rewrite(&input, case.id.clone(), RewriteOptions::default(), "corpus");
    if report.status != RewriteStatus::Normalized {
        return failure(
            &symbol,
            "not_normalized",
            format!("{} rewrote to {:?}, not Normalized", case.id, report.status),
        );
    }
    let wanted = case.expected_rule.as_deref().unwrap_or(&case.id);
    let fired = report
        .steps
        .iter()
        .find(|step| step.rule_id == wanted)
        .cloned();
    let Some(step) = fired else {
        return failure(
            &symbol,
            "rule_not_applied",
            format!("{} did not apply {wanted}", case.id),
        );
    };
    // The emitted revision must be the catalog's. Round 5 of the v0.1 review
    // found this stamped as a literal at seven call sites, so it is asserted per
    // rule rather than for the catalog as a whole.
    if step.rule_revision != rule.revision {
        return failure(
            &symbol,
            "revision_drift",
            format!(
                "{} emitted revision {} but the catalog names {}",
                case.id, step.rule_revision, rule.revision
            ),
        );
    }
    let Some(output) = report.output.as_ref() else {
        return failure(
            &symbol,
            "no_output",
            format!("{} normalized but produced no output document", case.id),
        );
    };

    let conformance = check_equivalence(
        &input,
        output,
        case.id.clone(),
        ConformanceOptions::default(),
    );
    let common = json!({
        "catalogSha256": conformance.catalog_sha256,
        "formulaSha256": observed_digest,
        "ruleRevision": step.rule_revision,
        "horizon": conformance.horizon,
        "traceLength": conformance.trace_length,
        "totalTraces": conformance.total_traces,
        "tracesChecked": conformance.traces_checked,
        "evaluatorRevision": conformance.evaluator_revision,
        "syntaxRevision": conformance.syntax_revision,
        "limitation": conformance.limitation,
    });
    match conformance.status {
        ConformanceStatus::Equivalent => Row {
            symbol,
            outcome: "pass",
            domain_outcome: "equivalent".to_owned(),
            detail: format!(
                "{} agreed on all {} traces of the complete bounded domain",
                case.id, conformance.traces_checked
            ),
            extra: common,
        },
        ConformanceStatus::Mismatch => {
            let mut extra = common;
            extra["counterexample"] = json!(conformance.counterexample);
            extra["originalVerdict"] = json!(conformance.original_verdict);
            extra["rewrittenVerdict"] = json!(conformance.rewritten_verdict);
            Row {
                symbol,
                outcome: "fail",
                domain_outcome: "mismatch".to_owned(),
                detail: format!("{} is not semantics-preserving on its own fixture", case.id),
                extra,
            }
        }
        ConformanceStatus::NonConclusive => Row {
            symbol,
            outcome: "not-computed",
            domain_outcome: non_conclusive_label(&conformance),
            detail: format!(
                "{} reached no verdict inside the declared bounded domain",
                case.id
            ),
            extra: common,
        },
    }
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().collect();
    let mut manifest_path = None;
    let mut index = 1;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--manifest" if index + 1 < arguments.len() => {
                manifest_path = Some(PathBuf::from(&arguments[index + 1]));
                index += 2;
            }
            other => {
                eprintln!("unknown argument {other}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(manifest_path) = manifest_path else {
        eprintln!("usage: rule_conformance --manifest <path>");
        return ExitCode::from(2);
    };

    let raw = match fs::read(&manifest_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {}: {error}", manifest_path.display());
            return ExitCode::from(2);
        }
    };
    let manifest: Manifest = match serde_json::from_slice(&raw) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("cannot parse {}: {error}", manifest_path.display());
            return ExitCode::from(2);
        }
    };
    if manifest.schema_version != SCHEMA {
        eprintln!("unsupported corpus schema {}", manifest.schema_version);
        return ExitCode::from(2);
    }
    if manifest.cases.is_empty() {
        eprintln!("the rule corpus declares no cases");
        return ExitCode::from(2);
    }

    // Exact set equality in both directions. A corpus that lists fewer rules
    // than the catalog would leave a rule unexercised while every listed row
    // still passed, and a corpus that lists more would assert evidence for a
    // rule that no longer exists.
    let document = catalog();
    let catalog_ids: BTreeSet<&str> = document.rules.iter().map(|rule| rule.id.as_str()).collect();
    let corpus_ids: BTreeSet<&str> = manifest.cases.iter().map(|case| case.id.as_str()).collect();
    if catalog_ids != corpus_ids {
        let missing: Vec<&&str> = catalog_ids.difference(&corpus_ids).collect();
        let extra: Vec<&&str> = corpus_ids.difference(&catalog_ids).collect();
        eprintln!(
            "the rule corpus and the catalog disagree; absent from the corpus: {missing:?}; \
             absent from the catalog: {extra:?}"
        );
        return ExitCode::from(2);
    }

    let mut failed = 0usize;
    for case in &manifest.cases {
        let rule = document
            .rules
            .iter()
            .find(|item| item.id == case.id)
            .expect("set equality was checked above");
        let row = evaluate(case, rule);
        if row.outcome == "fail" || row.outcome == "malformed" {
            failed += 1;
        }
        let mut object = json!({
            "protocol": PROTOCOL,
            "symbol": row.symbol,
            "outcome": row.outcome,
            "domainOutcome": row.domain_outcome,
            "traceIds": ["TC-013", "FR-001-AC-2", "FR-004-AC-1"],
            "disposition": case.disposition,
            "expectedOutcome": case.expected_outcome,
            "detail": row.detail,
        });
        if let (Some(target), Some(source)) = (object.as_object_mut(), row.extra.as_object()) {
            for (key, value) in source {
                target.insert(key.clone(), value.clone());
            }
        }
        println!("{object}");
    }

    // A producer that reported a failing row must exit non-zero. `make
    // conformance` is a CI gate, and a gate whose command always returns 0 is
    // not a gate. The rows remain the authority for the chain; this is the exit
    // status for the shell.
    if failed > 0 {
        eprintln!("{failed} catalog rule(s) disagreed with the corpus");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
