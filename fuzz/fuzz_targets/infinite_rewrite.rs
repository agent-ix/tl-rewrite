#![no_main]
#![forbid(unsafe_code)]

use libfuzzer_sys::fuzz_target;
use tl_oracle::{evaluate_documents, Limits as OracleLimits, OracleError};
use tl_rewrite::{replay_infinite, rewrite_infinite, RewriteBudgets, RewriteOptions};
use tl_syntax::{
    InfiniteClock, InfiniteFormulaDocument, InfiniteNodeKind, LassoTraceDocument, PartialValuation,
    PartialValue, PropositionId, SemanticProfile, SyntaxArtifactLimits, TraceObservation,
    ValuationEntry,
};

fn trace() -> LassoTraceDocument {
    let propositions = [PropositionId(0), PropositionId(1)];
    let row = |position, values: [PartialValue; 2]| TraceObservation {
        position,
        valuation: PartialValuation::new(
            "fuzz-map".to_owned(),
            &propositions,
            propositions
                .iter()
                .zip(values)
                .map(|(id, value)| ValuationEntry {
                    proposition: *id,
                    value,
                })
                .collect(),
        )
        .unwrap(),
    };
    LassoTraceDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        "fuzz-map".to_owned(),
        propositions.to_vec(),
        vec![row(0, [PartialValue::True, PartialValue::False])],
        vec![
            row(1, [PartialValue::False, PartialValue::True]),
            row(2, [PartialValue::Missing, PartialValue::False]),
        ],
    )
    .unwrap()
}

// Trace: TC-072, FR-020-AC-3, FR-046-AC-1.
fuzz_target!(|data: &[u8]| {
    if data.len() > 4096 {
        return;
    }
    // The public strict reader owns admission; do not normalize an arbitrary
    // JSON value through a separate decoder before this boundary.
    let Ok(input) =
        InfiniteFormulaDocument::from_json_bytes(data, SyntaxArtifactLimits::default())
    else {
        return;
    };
    if input.nodes().len() > 32
        || input.nodes().iter().any(|node| {
            matches!(
                node.kind,
                InfiniteNodeKind::Proposition { proposition } if proposition.0 > 1
            )
        })
    {
        return;
    }
    let options = RewriteOptions {
        budgets: RewriteBudgets {
            max_iterations: 8,
            max_nodes: 128,
            max_rule_applications: 256,
            max_work_units: 4096,
        },
        ..RewriteOptions::default()
    };
    let report = rewrite_infinite(&input, None, options, "fuzz/v1", 65_536);
    assert!(report.work_units <= options.budgets.max_work_units);
    assert!(report.iterations <= options.budgets.max_iterations);
    if !report.succeeded() {
        assert!(report.output.is_none() && report.steps.is_empty());
        return;
    }
    assert!(u64::try_from(report.steps.len()).unwrap() <= options.budgets.max_rule_applications);
    assert!(
        u32::try_from(report.output.as_ref().unwrap().nodes().len()).unwrap()
            <= options.budgets.max_nodes
    );
    assert!(replay_infinite(&input, None, &report, "fuzz/v1"));
    let output = report.output.as_ref().unwrap();
    let word = trace();
    let limits = OracleLimits {
        max_depth: 64,
        max_offset: 128,
        max_positions: 2048,
        max_completions: 16,
    };
    for position in 0..5 {
        let before = evaluate_documents(&input, &word, input.root(), &[], position, limits);
        let after = evaluate_documents(output, &word, output.root(), &[], position, limits);
        match before {
            Ok(expected) => assert_eq!(after, Ok(expected), "position={position}"),
            Err(OracleError::ResourceIncomplete | OracleError::NoPeriodicFixedPoint) => {}
            Err(error) => panic!("owner-admitted input refused by oracle: {error:?}"),
        }
    }
});
