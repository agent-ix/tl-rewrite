use std::collections::BTreeSet;

use serde::Serialize;
use sha2::{Digest, Sha256};
use tl_rewrite::{catalog, ProvenanceKind, RuleDisposition, WEST_REVISION};

fn digest<T: Serialize>(value: &T) -> String {
    Sha256::digest(serde_json::to_vec(value).unwrap())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

// Trace: TC-001, FR-001-AC-1, NFR-002-AC-1, StR-001-VC-1
#[test]
fn enabled_catalog_metadata_is_unique_and_complete() {
    let catalog = catalog();
    let mut ids = BTreeSet::new();
    assert!(catalog.rules.len() >= 30);
    for rule in &catalog.rules {
        assert!(ids.insert(&rule.id));
        assert_eq!(rule.revision, 1);
        assert!(!rule.precondition.is_empty());
        assert_eq!(rule.semantic_profiles, ["mltl.closed-trace/v1"]);
        assert!(!rule.provenance.uri.is_empty());
        assert!(!rule.provenance.locator.is_empty());
        assert!(!rule.provenance.statement.is_empty());
        if rule.disposition == RuleDisposition::Enabled {
            assert_eq!(rule.provenance.kind, ProvenanceKind::StatedDerivation);
            assert!(rule.exclusion_reason.is_none());
        }
    }
}

// Trace: TC-002, FR-001-AC-1, NFR-002-AC-1
#[test]
fn primary_source_candidates_remain_explicitly_excluded() {
    assert_eq!(WEST_REVISION.len(), 40);
    let excluded = catalog()
        .rules
        .into_iter()
        .filter(|rule| rule.disposition == RuleDisposition::Excluded)
        .collect::<Vec<_>>();
    assert_eq!(excluded.len(), 2);
    for rule in excluded {
        assert!(rule.id.starts_with("west.nested-"));
        assert_eq!(rule.provenance.kind, ProvenanceKind::PrimarySource);
        assert!(rule.provenance.uri.contains("10.1007"));
        assert!(rule.exclusion_reason.unwrap().contains("growth"));
    }
}

// Trace: TC-004, FR-001-AC-3, NFR-001-AC-1
#[test]
fn catalog_digest_changes_with_rule_revision_or_order() {
    let catalog = catalog();
    assert_eq!(catalog.catalog_sha256, digest(&catalog.rules));
    let mut revised = catalog.rules.clone();
    revised[0].revision += 1;
    assert_ne!(catalog.catalog_sha256, digest(&revised));
    revised = catalog.rules.clone();
    revised.swap(0, 1);
    assert_ne!(catalog.catalog_sha256, digest(&revised));
}
