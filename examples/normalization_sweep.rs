//! Sweep the normalization contract over a generated formula family (FR-006-AC-2).
//!
//! This is a producer. For every generated input it runs the real engine and
//! writes one JSON object to stdout. It computes no aggregate verdict and
//! retains nothing.
//!
//! Five properties are checked per formula, and each one is a separate way the
//! engine could be wrong:
//!
//! 1. **Determinism.** Two identical requests serialize to identical bytes.
//! 2. **Fixed point.** Rewriting the output again reports `Unchanged` and
//!    returns the same document, so `Normalized` means a fixed point was
//!    actually reached rather than a pass being abandoned.
//! 3. **Replay.** The recorded trace re-executes to the same report digest.
//! 4. **Structural validity and profile preservation.** The output still
//!    validates under `tl-syntax` and carries the input's profile.
//! 5. **Fail-closed budgets.** The same input under a zero work-unit budget
//!    returns `BudgetExhausted` with no output, so refusal is exercised on every
//!    formula rather than on one hand-picked one.
//!
//! A formula that trips none of these is `pass`. A formula that trips one is
//! `fail`, and the row names which. There is no third answer, because every
//! property here is decidable on the spot — nothing in this sweep can legitimately
//! reach no verdict, and a producer that emitted `not-computed` for a decidable
//! property would be hiding a defect behind a non-conclusive word.

use std::process::ExitCode;

use serde_json::json;
use tl_rewrite::{
    replay, rewrite, BudgetKind, ReplayStatus, RewriteBudgets, RewriteOptions, RewriteStatus,
};
use tl_syntax::{
    FormulaDocument, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile,
};

const PROTOCOL: &str = "tl-rewrite.normalization-sweep/v1";

fn leaf(index: usize) -> Node {
    match index {
        0 => Node::new(NodeKind::False),
        1 => Node::new(NodeKind::True),
        other => Node::new(NodeKind::Proposition {
            proposition: PropositionId((other - 2) as u32),
        }),
    }
}

fn binary(operator: usize, left: NodeId, right: NodeId, interval: Interval) -> NodeKind {
    match operator {
        0 => NodeKind::And { left, right },
        1 => NodeKind::Or { left, right },
        2 => NodeKind::Implies { left, right },
        3 => NodeKind::Equivalent { left, right },
        4 => NodeKind::Until {
            interval,
            left,
            right,
        },
        _ => NodeKind::Release {
            interval,
            left,
            right,
        },
    }
}

fn unary(operator: usize, operand: NodeId, interval: Interval) -> NodeKind {
    match operator {
        0 => NodeKind::Not { operand },
        1 => NodeKind::Future { interval, operand },
        _ => NodeKind::Globally { interval, operand },
    }
}

/// The generated domain: every binary operator over every leaf pair at three
/// intervals, each wrapped once by every unary operator. Intervals include lower
/// bounds above zero deliberately — the `a = 0` slice is where an earlier review
/// found the whole corpus had been sitting.
fn family() -> Vec<(String, FormulaDocument)> {
    let intervals = [
        Interval::new(0, 0).unwrap(),
        Interval::new(0, 2).unwrap(),
        Interval::new(1, 3).unwrap(),
    ];
    let mut generated = Vec::new();
    for (interval_index, interval) in intervals.into_iter().enumerate() {
        for operator in 0..6 {
            for left in 0..4 {
                for right in 0..4 {
                    for wrapper in 0..3 {
                        let nodes = vec![
                            leaf(left),
                            leaf(right),
                            Node::new(binary(operator, NodeId(0), NodeId(1), interval)),
                            Node::new(unary(wrapper, NodeId(2), interval)),
                        ];
                        let Ok(document) =
                            FormulaDocument::new(SemanticProfile::ClosedTraceV1, NodeId(3), nodes)
                        else {
                            continue;
                        };
                        generated.push((
                            format!(
                                "sweep:i{interval_index}-o{operator}-l{left}-r{right}-w{wrapper}"
                            ),
                            document,
                        ));
                    }
                }
            }
        }
    }
    generated
}

fn check(id: &str, input: &FormulaDocument) -> Result<serde_json::Value, String> {
    let options = RewriteOptions::default();
    let first = rewrite(input, id, options, "sweep");
    let second = rewrite(input, id, options, "sweep");
    let left = serde_json::to_vec(&first).map_err(|error| error.to_string())?;
    let right = serde_json::to_vec(&second).map_err(|error| error.to_string())?;
    if left != right {
        return Err("two identical requests produced different report bytes".to_owned());
    }
    if !matches!(
        first.status,
        RewriteStatus::Normalized | RewriteStatus::Unchanged
    ) {
        return Err(format!(
            "a default-budget rewrite of a four-node formula reported {:?}",
            first.status
        ));
    }
    let Some(output) = first.output.as_ref() else {
        return Err("a successful rewrite carried no output document".to_owned());
    };
    output
        .validate()
        .map_err(|error| format!("the output does not validate: {error}"))?;
    if output.semantic_profile() != input.semantic_profile() {
        return Err("the output changed the semantic profile".to_owned());
    }
    let again = rewrite(output, id, options, "sweep");
    if again.status != RewriteStatus::Unchanged {
        return Err(format!(
            "the output is not a fixed point: re-running reported {:?}",
            again.status
        ));
    }
    if again.output.as_ref() != Some(output) {
        return Err("re-running the output returned a different document".to_owned());
    }
    let replayed = replay(input, &first);
    if replayed.status != ReplayStatus::Verified {
        return Err("the recorded trace did not replay".to_owned());
    }
    let starved = rewrite(
        input,
        id,
        RewriteOptions {
            budgets: RewriteBudgets {
                max_work_units: 0,
                ..RewriteBudgets::default()
            },
            ..options
        },
        "sweep",
    );
    if starved.status != RewriteStatus::BudgetExhausted
        || starved.exhausted_budget != Some(BudgetKind::WorkUnits)
        || starved.output.is_some()
    {
        return Err(format!(
            "a zero work-unit budget reported {:?}/{:?} with output present: {}",
            starved.status,
            starved.exhausted_budget,
            starved.output.is_some()
        ));
    }
    Ok(json!({
        "status": format!("{:?}", first.status),
        "iterations": first.iterations,
        "workUnits": first.work_units,
        "ruleApplications": first.rule_applications,
        "steps": first.steps.len(),
        "outputNodes": output.nodes().len(),
        "requestSha256": first.request_sha256,
        "outputSha256": first.output_sha256,
        "replaySha256": replayed.observed_report_sha256,
    }))
}

fn main() -> ExitCode {
    let generated = family();
    if generated.is_empty() {
        eprintln!("the generated family is empty; there is nothing to sweep");
        return ExitCode::from(2);
    }
    let mut failed = 0usize;
    for (id, document) in &generated {
        let row = match check(id, document) {
            Ok(extra) => json!({
                "protocol": PROTOCOL,
                "symbol": id,
                "outcome": "pass",
                "domainOutcome": "normalized",
                "traceIds": ["TC-005", "TC-008", "TC-010", "FR-002-AC-1", "FR-002-AC-3"],
                "detail": "deterministic, fixed point, replayable, valid, and fail-closed",
                "properties": extra,
            }),
            Err(detail) => {
                failed += 1;
                json!({
                    "protocol": PROTOCOL,
                    "symbol": id,
                    "outcome": "fail",
                    "domainOutcome": "normalization_defect",
                    "traceIds": ["TC-005", "TC-008", "TC-010", "FR-002-AC-1", "FR-002-AC-3"],
                    "detail": detail,
                })
            }
        };
        println!("{row}");
    }
    if failed > 0 {
        eprintln!("{failed} generated formula(s) broke the normalization contract");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
