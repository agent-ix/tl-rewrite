use serde::{Deserialize, Serialize};

use crate::hash::sha256_json;

/// Semantic category used for catalog review and filtering.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleClass {
    /// Canonicalizes operator vocabulary.
    Normalization,
    /// Removes a redundant construct.
    Simplification,
    /// Pushes negation through a declared dual.
    Negation,
    /// Re-expresses a bounded temporal operator.
    Temporal,
}

/// Whether a catalog entry is executable in the v1 engine.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleDisposition {
    /// Rule is implemented and ordered in the v1 engine.
    Enabled,
    /// Rule is retained for review but cannot execute.
    Excluded,
}

/// Kind of semantic authority named by a rule.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceKind {
    /// Direct derivation from the pinned finite-trace semantic definitions.
    StatedDerivation,
    /// Published primary source with a stable locator.
    PrimarySource,
}

/// Reviewable semantic provenance for one rule.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Provenance {
    /// Authority class.
    pub kind: ProvenanceKind,
    /// Stable source or derivation URI.
    pub uri: String,
    /// Section, theorem, or derivation identifier.
    pub locator: String,
    /// Concise semantic justification.
    pub statement: String,
}

/// One immutable catalog entry.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuleDefinition {
    /// Stable rule identity.
    pub id: String,
    /// Revision of this rule's semantics and applicability metadata.
    pub revision: u32,
    /// Review category.
    pub class: RuleClass,
    /// Executable or retained-but-excluded disposition.
    pub disposition: RuleDisposition,
    /// Exact supported semantic-profile wire identities.
    pub semantic_profiles: Vec<String>,
    /// Complete applicability condition.
    pub precondition: String,
    /// Semantic authority.
    pub provenance: Provenance,
    /// Reason for exclusion, absent for enabled rules.
    pub exclusion_reason: Option<String>,
}

/// Versioned catalog document with a digest over ordered rule definitions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CatalogDocument {
    /// Wire schema identity.
    pub schema_version: String,
    /// Stable human-facing catalog identity.
    pub catalog_version: String,
    /// SHA-256 of the canonical ordered `rules` JSON value.
    pub catalog_sha256: String,
    /// Rules in exact first-match priority order.
    pub rules: Vec<RuleDefinition>,
}

fn profiles() -> Vec<String> {
    vec!["mltl.closed-trace/v1".to_owned()]
}

fn derived(
    id: &str,
    class: RuleClass,
    precondition: &str,
    locator: &str,
    statement: &str,
) -> RuleDefinition {
    RuleDefinition {
        id: id.to_owned(),
        revision: 1,
        class,
        disposition: RuleDisposition::Enabled,
        semantic_profiles: profiles(),
        precondition: precondition.to_owned(),
        provenance: Provenance {
            kind: ProvenanceKind::StatedDerivation,
            uri: "ix://agent-ix/tl-rewrite/DER-001".to_owned(),
            locator: locator.to_owned(),
            statement: statement.to_owned(),
        },
        exclusion_reason: None,
    }
}

fn excluded_primary(id: &str, precondition: &str, statement: &str, reason: &str) -> RuleDefinition {
    RuleDefinition {
        id: id.to_owned(),
        revision: 1,
        class: RuleClass::Temporal,
        disposition: RuleDisposition::Excluded,
        semantic_profiles: profiles(),
        precondition: precondition.to_owned(),
        provenance: Provenance {
            kind: ProvenanceKind::PrimarySource,
            uri: "https://doi.org/10.1007/978-3-031-47705-8_15".to_owned(),
            locator: "Theorem 3 (Nested Until and Release Rewriting Theorem)".to_owned(),
            statement: statement.to_owned(),
        },
        exclusion_reason: Some(reason.to_owned()),
    }
}

fn definitions() -> Vec<RuleDefinition> {
    use RuleClass::{Negation, Normalization, Simplification, Temporal};

    let specifications = [
        ("bool.not.false", Simplification, "operand is false", "DER-001-B01", "classical negation maps false to true"),
        ("bool.not.true", Simplification, "operand is true", "DER-001-B02", "classical negation maps true to false"),
        ("bool.not.double", Normalization, "operand is a negation", "DER-001-B03", "Boolean involution removes two negations"),
        ("neg.future.dual", Negation, "operand is bounded Future", "DER-001-N01", "not Future[a,b] p equals Globally[a,b] not p under three-valued duality"),
        ("neg.globally.dual", Negation, "operand is bounded Globally", "DER-001-N02", "not Globally[a,b] p equals Future[a,b] not p under three-valued duality"),
        ("neg.until.dual", Negation, "operand is bounded Until", "DER-001-N03", "not (p Until[a,b] q) equals (not p) Release[a,b] (not q) by the pinned Release definition"),
        ("neg.release.dual", Negation, "operand is bounded Release", "DER-001-N04", "not (p Release[a,b] q) equals (not p) Until[a,b] (not q) by the pinned Release definition"),
        ("bool.and.false-left", Simplification, "left operand is false", "DER-001-B04", "false and p is false"),
        ("bool.and.false-right", Simplification, "right operand is false", "DER-001-B05", "p and false is false"),
        ("bool.and.true-left", Simplification, "left operand is true", "DER-001-B06", "true and p is p"),
        ("bool.and.true-right", Simplification, "right operand is true", "DER-001-B07", "p and true is p"),
        ("bool.and.idempotent", Simplification, "both operands identify the same subtree", "DER-001-B08", "p and p is p"),
        ("bool.or.true-left", Simplification, "left operand is true", "DER-001-B09", "true or p is true"),
        ("bool.or.true-right", Simplification, "right operand is true", "DER-001-B10", "p or true is true"),
        ("bool.or.false-left", Simplification, "left operand is false", "DER-001-B11", "false or p is p"),
        ("bool.or.false-right", Simplification, "right operand is false", "DER-001-B12", "p or false is p"),
        ("bool.or.idempotent", Simplification, "both operands identify the same subtree", "DER-001-B13", "p or p is p"),
        ("bool.implies.false-left", Simplification, "antecedent is false", "DER-001-B14", "false implies p is true"),
        ("bool.implies.true-left", Simplification, "antecedent is true", "DER-001-B15", "true implies p is p"),
        ("bool.implies.true-right", Simplification, "consequent is true", "DER-001-B16", "p implies true is true"),
        ("bool.implies.false-right", Normalization, "consequent is false", "DER-001-B17", "p implies false is not p"),
        ("bool.implies.reflexive", Simplification, "both operands identify the same subtree", "DER-001-B18", "p implies p is true"),
        ("bool.implies.eliminate", Normalization, "no earlier implication simplification applies", "DER-001-B19", "p implies q equals not p or q"),
        ("bool.equivalent.reflexive", Simplification, "both operands identify the same subtree", "DER-001-B20", "p equivalent p is true"),
        ("bool.equivalent.true-left", Simplification, "left operand is true", "DER-001-B21", "true equivalent p is p"),
        ("bool.equivalent.true-right", Simplification, "right operand is true", "DER-001-B22", "p equivalent true is p"),
        ("bool.equivalent.false-left", Normalization, "left operand is false", "DER-001-B23", "false equivalent p is not p"),
        ("bool.equivalent.false-right", Normalization, "right operand is false", "DER-001-B24", "p equivalent false is not p"),
        ("temporal.future.singleton", Temporal, "interval is [0,0]", "DER-001-T01", "Future[0,0] p quantifies only the current instant and equals p"),
        ("temporal.globally.singleton", Temporal, "interval is [0,0]", "DER-001-T02", "Globally[0,0] p quantifies only the current instant and equals p"),
        ("temporal.until.singleton", Temporal, "interval is [0,0]", "DER-001-T03", "p Until[0,0] q has the current instant as its only witness and equals q"),
        ("temporal.release.singleton", Temporal, "interval is [0,0]", "DER-001-T04", "p Release[0,0] q is the dual singleton and equals q"),
        ("temporal.future.false", Simplification, "operand is false", "DER-001-T05", "Future over constant false is false"),
        ("temporal.future.true", Simplification, "operand is true", "DER-001-T06", "every non-empty inclusive interval has a true witness"),
        ("temporal.globally.false", Simplification, "operand is false", "DER-001-T07", "every non-empty inclusive interval contains a false value"),
        ("temporal.globally.true", Simplification, "operand is true", "DER-001-T08", "Globally over constant true is true"),
        ("temporal.until.true-left", Temporal, "left operand is true", "DER-001-T09", "true Until[a,b] q equals Future[a,b] q"),
        ("temporal.release.false-left", Temporal, "left operand is false", "DER-001-T10", "false Release[a,b] q equals Globally[a,b] q by Until duality"),
    ];
    let mut rules = specifications
        .into_iter()
        .map(|(id, class, precondition, locator, statement)| {
            derived(id, class, precondition, locator, statement)
        })
        .collect::<Vec<_>>();
    let exclusion = "excluded from v1 execution until interval decomposition policy and worst-case graph-growth evidence are approved";
    rules.push(excluded_primary(
        "west.nested-until-right",
        "nonnegative a,b,c with checked b+c and well-formed NNF operands",
        "p Until[a,b+c] q equals p Until[a,b] (p Until[0,c] q)",
        exclusion,
    ));
    rules.push(excluded_primary(
        "west.nested-release-right",
        "nonnegative a,b,c with checked b+c and well-formed NNF operands",
        "p Release[a,b+c] q equals p Release[a,b] (p Release[0,c] q)",
        exclusion,
    ));
    rules
}

/// Returns the immutable v1 catalog in exact engine priority order.
///
/// Implements: FR-001
pub fn catalog() -> CatalogDocument {
    let rules = definitions();
    CatalogDocument {
        schema_version: "tl-rewrite.catalog/v1".to_owned(),
        catalog_version: "tl-rewrite-rules/v1".to_owned(),
        catalog_sha256: sha256_json(&rules),
        rules,
    }
}
