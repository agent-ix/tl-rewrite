#![cfg(feature = "infinite-trace")]

//! Replays the syntax owner's pinned cases across the parser, rewriter, and
//! evaluator boundaries. Negative owner cases stay refusals at their owner.

use std::{fs, path::Path};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use tl_mltl::infinite::{Disposition, EvaluationLimit};
use tl_parse::{format_clean_ascii_v4, parse_clean_ascii_v4, FormatLimits, ParseLimits};
use tl_rewrite::{
    check_infinite_rewrite, rewrite_infinite, InfiniteConformanceReason, InfiniteConformanceStatus,
    RewriteOptions,
};
use tl_syntax::{
    InfiniteClock, InfiniteFormulaDocument, LassoTraceDocument, PartialValuation, PartialValue,
    PropositionEntry, PropositionId, PropositionMapDocument, SemanticProfile, TraceObservation,
    ValuationEntry, CORPUS_DIR,
};

#[derive(Deserialize)]
struct CorpusTrace {
    schema_version: String,
    semantic_profile: String,
    clock: String,
    proposition_map: Vec<PropositionEntry>,
    prefix: Vec<CorpusObservation>,
    #[serde(rename = "loop")]
    loop_observations: Vec<CorpusObservation>,
}

#[derive(Deserialize)]
struct CorpusObservation {
    position: u32,
    valuation: Vec<CorpusValue>,
}

#[derive(Deserialize)]
struct CorpusValue {
    proposition: PropositionId,
    state: PartialValue,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn observations(
    rows: Vec<CorpusObservation>,
    map_id: &str,
    propositions: &[PropositionId],
) -> Vec<TraceObservation> {
    rows.into_iter()
        .map(|row| TraceObservation {
            position: row.position,
            valuation: PartialValuation::new(
                map_id.to_owned(),
                propositions,
                row.valuation
                    .into_iter()
                    .map(|cell| ValuationEntry {
                        proposition: cell.proposition,
                        value: cell.state,
                    })
                    .collect(),
            )
            .unwrap(),
        })
        .collect()
}

// Trace: TC-173, TC-174; FR-030-AC-1/2, FR-019-AC-3, FR-020-AC-1.
#[test]
fn pinned_owner_cases_cross_parser_rewriter_and_provider() {
    let root = Path::new(CORPUS_DIR).join("infinite-trace");
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["corpus"], "tl-syntax.infinite-trace-corpus/v1");
    let sums = fs::read_to_string(root.join("SHA256SUMS")).unwrap();
    for pin in manifest["files"].as_array().unwrap() {
        let name = pin["path"].as_str().unwrap();
        let actual = digest(&fs::read(root.join(name)).unwrap());
        assert_eq!(actual, pin["sha256"], "{name}");
        assert!(sums.contains(&format!("{actual}  {name}\n")), "{name}");
    }
    let manifest_digest = digest(&fs::read(root.join("manifest.json")).unwrap());
    assert!(sums.contains(&format!("{manifest_digest}  manifest.json\n")));
    let corpus: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("cases.json")).unwrap()).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(
        cases.len(),
        manifest["case_count"].as_u64().unwrap() as usize
    );
    assert_eq!(cases.len(), 13);

    let mut compared = 0;
    let mut refused = 0;
    for case in cases {
        let id = case["id"].as_str().unwrap();
        let axis = case["expected"]["axis"].as_str();
        let formula: Result<InfiniteFormulaDocument, _> =
            serde_json::from_value(case["formula"].clone());
        if matches!(axis, Some("profile" | "operator")) {
            assert!(formula.is_err(), "{id}: owner admitted forbidden formula");
            refused += 1;
            continue;
        }
        let formula = formula.unwrap_or_else(|error| panic!("{id}: {error}"));
        let wire: CorpusTrace = serde_json::from_value(case["trace"].clone()).unwrap();
        assert_eq!(wire.schema_version, tl_syntax::LASSO_TRACE_V1, "{id}");
        assert_eq!(wire.semantic_profile, "mltl.infinite-trace/v1", "{id}");
        if axis == Some("clock") {
            assert_ne!(wire.clock, "event_position", "{id}");
            refused += 1;
            continue;
        }
        assert_eq!(wire.clock, "event_position", "{id}");
        if axis == Some("fairness") {
            assert!(wire.loop_observations.is_empty(), "{id}");
            refused += 1;
            continue;
        }
        let roots = serde_json::from_value(case["fairness"]["roots"].clone()).unwrap();
        let formula_identity = formula.content_identity().unwrap();
        let fairness = tl_syntax::FairnessPremisesDocument::new(
            &formula,
            formula_identity,
            InfiniteClock::EventPosition,
            roots,
        )
        .unwrap();
        let formatted = format_clean_ascii_v4(&formula, Some(&fairness), FormatLimits::default());
        let text = formatted
            .text
            .unwrap_or_else(|| panic!("{id}: formatter refused {:?}", formatted.error));
        let parsed = parse_clean_ascii_v4(
            &text,
            SemanticProfile::InfiniteTraceV1,
            "event_position",
            ParseLimits::default(),
        );
        let input = parsed
            .document
            .as_ref()
            .unwrap_or_else(|| panic!("{id}: parser refused {:?}", parsed.diagnostics));
        let input_fairness = parsed.fairness.as_ref();
        assert_eq!(
            input_fairness
                .map(|premises| premises.roots().len())
                .unwrap_or(0),
            fairness.roots().len(),
            "{id}"
        );
        let reformatted = format_clean_ascii_v4(input, input_fairness, FormatLimits::default());
        assert_eq!(reformatted.text.as_deref(), Some(text.as_str()), "{id}");

        let map = PropositionMapDocument::new(wire.proposition_map).unwrap();
        let map_id = map.content_identity().unwrap();
        let propositions: Vec<_> = map.propositions().iter().map(|entry| entry.id).collect();
        let prefix = observations(wire.prefix, &map_id, &propositions);
        let loop_observations = observations(wire.loop_observations, &map_id, &propositions);
        let trace = LassoTraceDocument::new(
            SemanticProfile::InfiniteTraceV1,
            InfiniteClock::EventPosition,
            map_id,
            propositions,
            prefix,
            loop_observations,
        )
        .unwrap();
        let rewritten = rewrite_infinite(
            input,
            input_fairness,
            RewriteOptions::default(),
            "owner-corpus/v1",
            1_000_000,
        );
        assert!(rewritten.succeeded(), "{id}: {:?}", rewritten.failure);
        let checked = check_infinite_rewrite(
            input,
            input_fairness,
            &trace,
            &rewritten,
            case["anchor"].as_u64().unwrap(),
            "owner-corpus/v1",
            EvaluationLimit::default(),
        );
        let expected = case["expected"]["value"].as_str().unwrap();
        if id == "conflicting-value-is-inconclusive" {
            assert_eq!(expected, "inconclusive");
            assert_eq!(checked.status, InfiniteConformanceStatus::NonConclusive);
            assert_eq!(
                checked.reason,
                Some(InfiniteConformanceReason::ConflictingObservation)
            );
            assert!(checked.before.is_none() && checked.after.is_none());
        } else {
            assert_eq!(
                checked.status,
                InfiniteConformanceStatus::Equivalent,
                "{id}"
            );
            let result = checked.before.as_ref().unwrap();
            let actual = match result.disposition {
                Disposition::Proved => "proved",
                Disposition::Refuted => "refuted",
                Disposition::Inconclusive => "inconclusive",
                Disposition::Unsupported => "unsupported",
                Disposition::Failed => "failed",
            };
            assert_eq!(actual, expected, "{id}");
        }
        compared += 1;
    }
    assert_eq!(compared, 9);
    assert_eq!(refused, 4);
}
