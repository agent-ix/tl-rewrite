mod common;

use common::{document, proposition};
use tl_rewrite::{
    catalog, check_equivalence, rewrite, ConformanceOptions, ConformanceStatus, RewriteOptions,
    RewriteStatus, RuleDisposition,
};
use tl_syntax::{Interval, Node, NodeId, NodeKind, SemanticProfile};

fn unary(kind: impl FnOnce(NodeId) -> NodeKind, leaf: Node) -> tl_syntax::FormulaDocument {
    document(
        SemanticProfile::ClosedTraceV1,
        vec![leaf, Node::new(kind(NodeId(0)))],
    )
}

fn binary(
    kind: impl FnOnce(NodeId, NodeId) -> NodeKind,
    left: Node,
    right: Node,
) -> tl_syntax::FormulaDocument {
    document(
        SemanticProfile::ClosedTraceV1,
        vec![left, right, Node::new(kind(NodeId(0), NodeId(1)))],
    )
}

fn same(kind: impl FnOnce(NodeId, NodeId) -> NodeKind) -> tl_syntax::FormulaDocument {
    document(
        SemanticProfile::ClosedTraceV1,
        vec![proposition(0), Node::new(kind(NodeId(0), NodeId(0)))],
    )
}

fn fixture(id: &str) -> tl_syntax::FormulaDocument {
    let interval = Interval::new(1, 2).unwrap();
    let singleton = Interval::new(0, 0).unwrap();
    match id {
        "bool.not.false" => unary(
            |operand| NodeKind::Not { operand },
            Node::new(NodeKind::False),
        ),
        "bool.not.true" => unary(
            |operand| NodeKind::Not { operand },
            Node::new(NodeKind::True),
        ),
        "bool.not.double" => document(
            SemanticProfile::ClosedTraceV1,
            vec![
                proposition(0),
                Node::new(NodeKind::Not { operand: NodeId(0) }),
                Node::new(NodeKind::Not { operand: NodeId(1) }),
            ],
        ),
        "neg.future.dual" => document(
            SemanticProfile::ClosedTraceV1,
            vec![
                proposition(0),
                Node::new(NodeKind::Future {
                    interval,
                    operand: NodeId(0),
                }),
                Node::new(NodeKind::Not { operand: NodeId(1) }),
            ],
        ),
        "neg.globally.dual" => document(
            SemanticProfile::ClosedTraceV1,
            vec![
                proposition(0),
                Node::new(NodeKind::Globally {
                    interval,
                    operand: NodeId(0),
                }),
                Node::new(NodeKind::Not { operand: NodeId(1) }),
            ],
        ),
        "neg.until.dual" | "neg.release.dual" => {
            let temporal = if id == "neg.until.dual" {
                NodeKind::Until {
                    interval,
                    left: NodeId(0),
                    right: NodeId(1),
                }
            } else {
                NodeKind::Release {
                    interval,
                    left: NodeId(0),
                    right: NodeId(1),
                }
            };
            document(
                SemanticProfile::ClosedTraceV1,
                vec![
                    proposition(0),
                    proposition(1),
                    Node::new(temporal),
                    Node::new(NodeKind::Not { operand: NodeId(2) }),
                ],
            )
        }
        "bool.and.false-left" => binary(
            |left, right| NodeKind::And { left, right },
            Node::new(NodeKind::False),
            proposition(0),
        ),
        "bool.and.false-right" => binary(
            |left, right| NodeKind::And { left, right },
            proposition(0),
            Node::new(NodeKind::False),
        ),
        "bool.and.true-left" => binary(
            |left, right| NodeKind::And { left, right },
            Node::new(NodeKind::True),
            proposition(0),
        ),
        "bool.and.true-right" => binary(
            |left, right| NodeKind::And { left, right },
            proposition(0),
            Node::new(NodeKind::True),
        ),
        "bool.and.idempotent" => same(|left, right| NodeKind::And { left, right }),
        "bool.or.true-left" => binary(
            |left, right| NodeKind::Or { left, right },
            Node::new(NodeKind::True),
            proposition(0),
        ),
        "bool.or.true-right" => binary(
            |left, right| NodeKind::Or { left, right },
            proposition(0),
            Node::new(NodeKind::True),
        ),
        "bool.or.false-left" => binary(
            |left, right| NodeKind::Or { left, right },
            Node::new(NodeKind::False),
            proposition(0),
        ),
        "bool.or.false-right" => binary(
            |left, right| NodeKind::Or { left, right },
            proposition(0),
            Node::new(NodeKind::False),
        ),
        "bool.or.idempotent" => same(|left, right| NodeKind::Or { left, right }),
        "bool.implies.false-left" => binary(
            |left, right| NodeKind::Implies { left, right },
            Node::new(NodeKind::False),
            proposition(0),
        ),
        "bool.implies.true-left" => binary(
            |left, right| NodeKind::Implies { left, right },
            Node::new(NodeKind::True),
            proposition(0),
        ),
        "bool.implies.true-right" => binary(
            |left, right| NodeKind::Implies { left, right },
            proposition(0),
            Node::new(NodeKind::True),
        ),
        "bool.implies.false-right" => binary(
            |left, right| NodeKind::Implies { left, right },
            proposition(0),
            Node::new(NodeKind::False),
        ),
        "bool.implies.reflexive" => same(|left, right| NodeKind::Implies { left, right }),
        "bool.implies.eliminate" => binary(
            |left, right| NodeKind::Implies { left, right },
            proposition(0),
            proposition(1),
        ),
        "bool.equivalent.reflexive" => same(|left, right| NodeKind::Equivalent { left, right }),
        "bool.equivalent.true-left" => binary(
            |left, right| NodeKind::Equivalent { left, right },
            Node::new(NodeKind::True),
            proposition(0),
        ),
        "bool.equivalent.true-right" => binary(
            |left, right| NodeKind::Equivalent { left, right },
            proposition(0),
            Node::new(NodeKind::True),
        ),
        "bool.equivalent.false-left" => binary(
            |left, right| NodeKind::Equivalent { left, right },
            Node::new(NodeKind::False),
            proposition(0),
        ),
        "bool.equivalent.false-right" => binary(
            |left, right| NodeKind::Equivalent { left, right },
            proposition(0),
            Node::new(NodeKind::False),
        ),
        "temporal.future.singleton" => unary(
            |operand| NodeKind::Future {
                interval: singleton,
                operand,
            },
            proposition(0),
        ),
        "temporal.globally.singleton" => unary(
            |operand| NodeKind::Globally {
                interval: singleton,
                operand,
            },
            proposition(0),
        ),
        "temporal.until.singleton" => binary(
            |left, right| NodeKind::Until {
                interval: singleton,
                left,
                right,
            },
            proposition(0),
            proposition(1),
        ),
        "temporal.release.singleton" => binary(
            |left, right| NodeKind::Release {
                interval: singleton,
                left,
                right,
            },
            proposition(0),
            proposition(1),
        ),
        "temporal.future.false" => unary(
            |operand| NodeKind::Future { interval, operand },
            Node::new(NodeKind::False),
        ),
        "temporal.future.true" => unary(
            |operand| NodeKind::Future { interval, operand },
            Node::new(NodeKind::True),
        ),
        "temporal.globally.false" => unary(
            |operand| NodeKind::Globally { interval, operand },
            Node::new(NodeKind::False),
        ),
        "temporal.globally.true" => unary(
            |operand| NodeKind::Globally { interval, operand },
            Node::new(NodeKind::True),
        ),
        "temporal.until.true-left" => binary(
            |left, right| NodeKind::Until {
                interval,
                left,
                right,
            },
            Node::new(NodeKind::True),
            proposition(0),
        ),
        "temporal.release.false-left" => binary(
            |left, right| NodeKind::Release {
                interval,
                left,
                right,
            },
            Node::new(NodeKind::False),
            proposition(0),
        ),
        other => panic!("missing evidence fixture for enabled rule {other}"),
    }
}

// Trace: TC-013, FR-004-AC-1, FR-001-AC-2
#[test]
fn every_enabled_rule_has_positive_exhaustive_bounded_evidence() {
    let enabled = catalog()
        .rules
        .into_iter()
        .filter(|rule| rule.disposition == RuleDisposition::Enabled)
        .collect::<Vec<_>>();
    assert_eq!(enabled.len(), 38);
    for rule in enabled {
        let input = fixture(&rule.id);
        let rewritten = rewrite(&input, rule.id.clone(), RewriteOptions::default(), "source");
        assert_eq!(rewritten.status, RewriteStatus::Normalized, "{}", rule.id);
        assert!(
            rewritten.steps.iter().any(|step| step.rule_id == rule.id),
            "{} was not exercised",
            rule.id
        );
        assert!(
            rewritten
                .steps
                .iter()
                .any(|step| { step.rule_id == rule.id && step.rule_revision == rule.revision }),
            "{} did not emit its catalog revision",
            rule.id
        );
        let conformance = check_equivalence(
            &input,
            rewritten.output.as_ref().unwrap(),
            rule.id.clone(),
            ConformanceOptions::default(),
        );
        assert_eq!(
            conformance.status,
            ConformanceStatus::Equivalent,
            "{}",
            rule.id
        );
    }
}

// Trace: TC-022, FR-001-AC-2, FR-004-AC-3
#[test]
fn the_rule_corpus_is_the_constructed_fixtures_and_covers_the_whole_catalog() {
    // The corpus in `corpus/rules/manifest.json` is what `examples/rule_conformance.rs`
    // replays, and it is data rather than code. Data that nothing binds to the
    // reviewed construction is data anyone can edit, so both directions are
    // asserted here: every enabled rule's constructed fixture must serialize to
    // exactly the digest the corpus declares, and the corpus's case set must be
    // the catalog's rule set.
    //
    // This is the same shape as the WEST fixture-to-manifest binding in
    // tests/equivalence.rs, which an adversarial review of v0.1 attacked twice
    // and which held both times.
    use std::{collections::BTreeSet, fs, path::PathBuf};

    use serde::Deserialize;
    use sha2::{Digest, Sha256};

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Manifest {
        schema_version: String,
        cases: Vec<Case>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Case {
        id: String,
        disposition: String,
        #[serde(default)]
        formula_sha256: Option<String>,
        #[serde(default)]
        exclusion_reason: Option<String>,
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(root.join("corpus/rules/manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest.schema_version, "tl-rewrite.rule-corpus/v1");

    let document = catalog();
    let catalog_ids: BTreeSet<&str> = document.rules.iter().map(|rule| rule.id.as_str()).collect();
    let corpus_ids: BTreeSet<&str> = manifest.cases.iter().map(|case| case.id.as_str()).collect();
    assert_eq!(
        catalog_ids, corpus_ids,
        "the rule corpus and the catalog name different rule sets"
    );

    let mut enabled = 0usize;
    let mut excluded = 0usize;
    for case in &manifest.cases {
        let rule = document
            .rules
            .iter()
            .find(|item| item.id == case.id)
            .expect("set equality was asserted above");
        match case.disposition.as_str() {
            "enabled" => {
                enabled += 1;
                assert_eq!(rule.disposition, RuleDisposition::Enabled, "{}", case.id);
                let expected = case
                    .formula_sha256
                    .as_ref()
                    .unwrap_or_else(|| panic!("{} declares no formulaSha256", case.id));
                let constructed = fixture(&case.id);
                let observed: String = Sha256::digest(serde_json::to_vec(&constructed).unwrap())
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect();
                assert_eq!(
                    &observed, expected,
                    "{} in the corpus is not the fixture this test constructs",
                    case.id
                );
            }
            "excluded" => {
                excluded += 1;
                assert_eq!(rule.disposition, RuleDisposition::Excluded, "{}", case.id);
                assert_eq!(
                    case.exclusion_reason.as_deref(),
                    rule.exclusion_reason.as_deref(),
                    "{} exclusion reason drifted from the catalog",
                    case.id
                );
            }
            other => panic!("{} declares an unknown disposition {other}", case.id),
        }
    }
    assert_eq!(enabled, 38, "the corpus declares {enabled} enabled rules");
    assert_eq!(excluded, 2, "the corpus declares {excluded} excluded rules");
}
