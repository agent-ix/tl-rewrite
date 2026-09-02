//! Produce and independently replay the retained counterexample corpus (FR-006-AC-2, FR-006-AC-6).
//!
//! This is a producer. It runs the actual bounded comparison and the actual
//! pinned `tl-mltl` evaluator over the checked counterevidence corpus and writes
//! one JSON object per case to stdout. It computes no aggregate verdict and
//! retains nothing.
//!
//! # Why a counterexample must not become a boolean
//!
//! `check_equivalence` reports `Mismatch` together with the first disagreeing
//! trace in stable enumeration order and both verdicts on it. A pipeline that
//! keeps only "mismatched: true" has thrown away the entire content of the
//! finding: which trace, which instant, and which way each side answered. Once
//! that is gone, a comparison that stopped enumerating early and a comparison
//! that found a genuine divergence look identical.
//!
//! So every mismatch row here carries the witness itself, and the witness is
//! then *replayed*: the trace is rebuilt from the reported instants and handed
//! straight to `tl_mltl::evaluate_closed` against both documents, outside the
//! enumeration that produced it. The row records both replayed verdicts and
//! requires them to differ and to equal the reported pair. A recorded
//! counterexample that does not actually separate the two formulas is a `fail`,
//! not a pass with a decorative field.
//!
//! # The outcome vocabulary
//!
//! `pass` — the case reached its declared outcome. For a mismatch case that
//! means a replayed witness; for a non-conclusive case it means the comparison
//! declined for the declared reason. Declining correctly is a discharged
//! obligation, so it is a pass of the *proof* while `domainOutcome` keeps the
//! exact bounded-domain reason.
//!
//! `malformed` — the case's document does not decode, and the case declared
//! that it would not. The engine naming the state instead of producing an answer
//! is the obligation, so the word stays in the outcome column.
//!
//! `not-computed` — a case that unexpectedly reached no verdict. That is a real
//! not-computed and it is meant to poison the proof.
//!
//! `unavailable` — a declared input that is absent. No corpus case produces this
//! on the green path; it exists so that a corpus file which disappears is
//! reported rather than skipped.
//!
//! `fail` — the observed outcome is not the declared one.

use std::{fs, path::PathBuf, process::ExitCode};

use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tl_mltl::{evaluate_closed, EvaluationLimits, TruthValue};
use tl_rewrite::{check_equivalence, ConformanceOptions, ConformanceReport, ConformanceStatus};
use tl_syntax::{FormulaDocument, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

const PROTOCOL: &str = "tl-rewrite.counterexample-evidence/v1";
const SCHEMA: &str = "tl-rewrite.counterexample-corpus/v1";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    schema_version: String,
    #[serde(default)]
    purpose: String,
    #[serde(default)]
    oracle: String,
    #[serde(default)]
    counts: Value,
    #[serde(default)]
    unavailable_is_not_a_case: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    id: String,
    kind: String,
    expected_domain_outcome: String,
    #[serde(default)]
    original: Option<Value>,
    #[serde(default)]
    rewritten: Option<Value>,
    #[serde(default)]
    original_raw: Option<Value>,
    #[serde(default)]
    original_file: Option<String>,
    #[serde(default)]
    generator: Option<Generator>,
    #[serde(default)]
    options: Option<Options>,
    #[serde(default)]
    why: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Generator {
    kind: String,
    depth: u32,
    proposition: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    max_propositions: u32,
    max_horizon: u64,
    max_traces: u64,
}

struct Row {
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

fn fail(domain: &str, detail: String) -> Row {
    Row {
        outcome: "fail",
        domain_outcome: domain.to_owned(),
        detail,
        extra: json!({}),
    }
}

fn reason_label(report: &ConformanceReport) -> String {
    let reason = report
        .reason
        .and_then(|value| serde_json::to_value(value).ok())
        .and_then(|item| item.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unstated".to_owned());
    format!("non_conclusive:{reason}")
}

/// Build the declared negation chain. Declared in the corpus rather than inlined
/// because 514 nodes of literal JSON would bury the single fact the case is about.
fn generated(generator: &Generator) -> Option<FormulaDocument> {
    if generator.kind != "not-chain" {
        return None;
    }
    let mut nodes = vec![Node::new(NodeKind::Proposition {
        proposition: PropositionId(generator.proposition),
    })];
    for index in 0..generator.depth {
        nodes.push(Node::new(NodeKind::Not {
            operand: NodeId(index),
        }));
    }
    let root = NodeId((nodes.len() - 1) as u32);
    FormulaDocument::new(SemanticProfile::ClosedTraceV1, root, nodes).ok()
}

/// Re-evaluate a recorded counterexample outside the enumeration that found it.
///
/// This is the whole reason the witness is retained. `check_equivalence` walks a
/// complete bounded domain and reports the first disagreement; this rebuilds
/// only the reported trace and asks the pinned evaluator directly. If the two
/// answers agree, the recorded trace does not separate the formulas and the
/// mismatch is not backed by the thing it says backs it.
fn replay_witness(
    original: &FormulaDocument,
    rewritten: &FormulaDocument,
    trace: &[Vec<u32>],
) -> Result<(TruthValue, TruthValue), String> {
    let original_formula = original
        .validate()
        .map_err(|error| format!("the original document no longer validates: {error}"))?;
    let rewritten_formula = rewritten
        .validate()
        .map_err(|error| format!("the rewritten document no longer validates: {error}"))?;
    let candidate: Vec<Vec<PropositionId>> = trace
        .iter()
        .map(|instant| instant.iter().map(|id| PropositionId(*id)).collect())
        .collect();
    let left = evaluate_closed(
        original_formula,
        "witness-original",
        &candidate,
        "witness".to_owned(),
        EvaluationLimits::default(),
    )
    .map_err(|error| format!("the evaluator refused the witness for the original: {error:?}"))?;
    let right = evaluate_closed(
        rewritten_formula,
        "witness-rewritten",
        &candidate,
        "witness".to_owned(),
        EvaluationLimits::default(),
    )
    .map_err(|error| format!("the evaluator refused the witness for the rewrite: {error:?}"))?;
    Ok((left.verdict, right.verdict))
}

fn decode(value: &Value) -> Result<FormulaDocument, String> {
    serde_json::from_value(value.clone()).map_err(|error| error.to_string())
}

fn evaluate(root: &std::path::Path, case: &Case) -> Row {
    // A declared input file that is absent is unavailable, which is a different
    // fact from a comparison that ran and disagreed. It is reported, never
    // skipped: a case that quietly disappears and a case that agreed are
    // otherwise the same green.
    if let Some(relative) = case.original_file.as_ref() {
        if !root.join(relative).is_file() {
            return Row {
                outcome: "unavailable",
                domain_outcome: "absent_input".to_owned(),
                detail: format!("{} names {relative}, which is not present", case.id),
                extra: json!({ "declaredFile": relative }),
            };
        }
    }

    // The malformed class. The obligation is that the document does not decode
    // and that this is reported as such rather than being tolerated.
    if case.kind == "malformed" {
        let Some(raw) = case.original_raw.as_ref() else {
            return fail(
                "malformed_input",
                format!("{} declares kind malformed but no originalRaw", case.id),
            );
        };
        return match decode(raw) {
            Ok(_) => fail(
                "malformed_input",
                format!(
                    "{} declares a malformed document but it decoded successfully",
                    case.id
                ),
            ),
            Err(error) => Row {
                outcome: "malformed",
                domain_outcome: "malformed_input".to_owned(),
                detail: format!("{} was refused by the decoder: {error}", case.id),
                extra: json!({ "decoderMessage": error }),
            },
        };
    }

    let (original, rewritten) = if let Some(generator) = case.generator.as_ref() {
        let Some(document) = generated(generator) else {
            return fail(
                "generator_error",
                format!("{} declares an unusable generator", case.id),
            );
        };
        (document.clone(), document)
    } else {
        let (Some(left), Some(right)) = (case.original.as_ref(), case.rewritten.as_ref()) else {
            return fail(
                "malformed_input",
                format!("{} declares no formula pair", case.id),
            );
        };
        match (decode(left), decode(right)) {
            (Ok(left), Ok(right)) => (left, right),
            (Err(error), _) | (_, Err(error)) => {
                return fail(
                    "malformed_input",
                    format!("{} did not decode: {error}", case.id),
                )
            }
        }
    };

    let options = case
        .options
        .as_ref()
        .map(|item| ConformanceOptions {
            max_propositions: item.max_propositions,
            max_horizon: item.max_horizon,
            max_traces: item.max_traces,
        })
        .unwrap_or_default();

    let original_digest = serde_json::to_vec(&original).map(|bytes| digest(&bytes));
    let rewritten_digest = serde_json::to_vec(&rewritten).map(|bytes| digest(&bytes));
    let (Ok(original_digest), Ok(rewritten_digest)) = (original_digest, rewritten_digest) else {
        return fail(
            "malformed_input",
            format!("{} did not re-serialize", case.id),
        );
    };

    let report = check_equivalence(&original, &rewritten, case.id.clone(), options);
    let common = json!({
        "originalSha256": original_digest,
        "rewrittenSha256": rewritten_digest,
        "reportOriginalSha256": report.original_sha256,
        "reportRewrittenSha256": report.rewritten_sha256,
        "horizon": report.horizon,
        "traceLength": report.trace_length,
        "totalTraces": report.total_traces,
        "tracesChecked": report.traces_checked,
        "evaluatorRevision": report.evaluator_revision,
        "syntaxRevision": report.syntax_revision,
        "limitation": report.limitation,
    });

    match report.status {
        ConformanceStatus::Mismatch => {
            if case.kind != "mismatch" {
                return fail(
                    "mismatch",
                    format!(
                        "{} declares {} but the comparison found a counterexample",
                        case.id, case.expected_domain_outcome
                    ),
                );
            }
            let Some(trace) = report.counterexample.as_ref() else {
                return fail(
                    "mismatch",
                    format!(
                        "{} reported a mismatch with no counterexample; the finding has no witness",
                        case.id
                    ),
                );
            };
            if trace.is_empty() {
                return fail(
                    "mismatch",
                    format!("{} reported an empty counterexample trace", case.id),
                );
            }
            let (Some(reported_original), Some(reported_rewritten)) =
                (report.original_verdict, report.rewritten_verdict)
            else {
                return fail(
                    "mismatch",
                    format!("{} reported a counterexample with no verdict pair", case.id),
                );
            };
            let replayed = match replay_witness(&original, &rewritten, trace) {
                Ok(pair) => pair,
                Err(error) => return fail("mismatch", format!("{}: {error}", case.id)),
            };
            if replayed.0 == replayed.1 {
                return fail(
                    "mismatch",
                    format!(
                        "{} recorded a counterexample that does not separate the two formulas: \
                         both replay to {:?}",
                        case.id, replayed.0
                    ),
                );
            }
            if replayed.0 != reported_original || replayed.1 != reported_rewritten {
                return fail(
                    "mismatch",
                    format!(
                        "{} replayed to ({:?}, {:?}) but reported ({:?}, {:?})",
                        case.id, replayed.0, replayed.1, reported_original, reported_rewritten
                    ),
                );
            }
            let mut extra = common;
            extra["counterexample"] = json!(trace);
            extra["counterexampleInstants"] = json!(trace.len());
            extra["originalVerdict"] = json!(reported_original);
            extra["rewrittenVerdict"] = json!(reported_rewritten);
            extra["witnessReplayed"] = json!(true);
            extra["replayedOriginalVerdict"] = json!(replayed.0);
            extra["replayedRewrittenVerdict"] = json!(replayed.1);
            extra["counterexampleSha256"] = json!(digest(
                serde_json::to_vec(trace).unwrap_or_default().as_slice()
            ));
            Row {
                outcome: "pass",
                domain_outcome: "mismatch".to_owned(),
                detail: format!(
                    "{} is separated by a {}-instant witness, replayed outside the enumeration",
                    case.id,
                    trace.len()
                ),
                extra,
            }
        }
        ConformanceStatus::NonConclusive => {
            let observed = reason_label(&report);
            if case.kind != "non_conclusive" {
                return fail(
                    observed.as_str(),
                    format!(
                        "{} declares {} but the comparison reached no verdict ({observed})",
                        case.id, case.expected_domain_outcome
                    ),
                );
            }
            if observed != case.expected_domain_outcome {
                return fail(
                    observed.as_str(),
                    format!(
                        "{} declares {} but observed {observed}",
                        case.id, case.expected_domain_outcome
                    ),
                );
            }
            Row {
                outcome: "pass",
                domain_outcome: observed,
                detail: format!(
                    "{} declined inside the declared boundary, as the corpus requires",
                    case.id
                ),
                extra: common,
            }
        }
        ConformanceStatus::Equivalent => fail(
            "equivalent",
            format!(
                "{} declares {} but the comparison found the pair equivalent over {} traces",
                case.id, case.expected_domain_outcome, report.traces_checked
            ),
        ),
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
        eprintln!("usage: counterexample_evidence --manifest <path>");
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
    let _ = (
        &manifest.purpose,
        &manifest.oracle,
        &manifest.unavailable_is_not_a_case,
    );
    if manifest.schema_version != SCHEMA {
        eprintln!("unsupported corpus schema {}", manifest.schema_version);
        return ExitCode::from(2);
    }
    if manifest.cases.is_empty() {
        eprintln!("the counterexample corpus declares no cases");
        return ExitCode::from(2);
    }
    // The declared counts are the oracle the chain compares against. If the file
    // disagrees with itself there is no oracle at all, so that is an environment
    // error rather than a failing row.
    for (kind, declared) in [
        ("mismatch", "mismatch"),
        ("non_conclusive", "non_conclusive"),
        ("malformed", "malformed"),
    ] {
        let observed = manifest
            .cases
            .iter()
            .filter(|case| case.kind == kind)
            .count();
        let stated = manifest.counts.get(declared).and_then(Value::as_u64);
        if stated != Some(observed as u64) {
            eprintln!(
                "the corpus declares {stated:?} {kind} cases but lists {observed}; the count \
                 oracle disagrees with the cases it counts"
            );
            return ExitCode::from(2);
        }
    }

    let root = manifest_path
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut failed = 0usize;
    for case in &manifest.cases {
        let row = evaluate(&root, case);
        if row.outcome == "fail" {
            failed += 1;
        }
        let mut object = json!({
            "protocol": PROTOCOL,
            "symbol": format!("counterexample:{}", case.id),
            "outcome": row.outcome,
            "domainOutcome": row.domain_outcome,
            "traceIds": ["TC-014", "FR-004-AC-1", "FR-004-AC-2"],
            "declaredKind": case.kind,
            "expectedDomainOutcome": case.expected_domain_outcome,
            "detail": row.detail,
            "why": case.why,
        });
        if let (Some(target), Some(source)) = (object.as_object_mut(), row.extra.as_object()) {
            for (key, value) in source {
                target.insert(key.clone(), value.clone());
            }
        }
        println!("{object}");
    }
    if failed > 0 {
        eprintln!("{failed} counterexample case(s) disagreed with the corpus");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
