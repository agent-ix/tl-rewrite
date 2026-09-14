//! Deterministic rewrite-report replay and bounded canonical admission.

use tl_syntax::{FormulaDocument, RequirementContextDocument, SignalCatalogDocument};

use serde::{de::Error as _, Deserialize, Serialize};

use crate::{
    engine::{rewrite, rewrite_with_context},
    hash::sha256_json,
    RewriteReport,
};

pub use crate::report::{RecordLimits, RecordReadError, RecordReadErrorCode};

/// Replay comparison status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayStatus {
    /// Re-execution reproduced the complete report.
    Verified,
    /// Re-execution differed in at least one field.
    Mismatch,
}

/// Versioned replay result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplayReport {
    /// Wire schema identity.
    pub schema_version: String,
    /// Comparison result.
    pub status: ReplayStatus,
    /// Digest of the supplied report.
    pub expected_report_sha256: String,
    /// Digest of the freshly observed report.
    pub observed_report_sha256: String,
    /// Complete supplied signal-catalog identity for contextual v2 replay.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_catalog_sha256: Option<String>,
    /// Exact caller context for contextual v2 replay; `Some(None)` encodes JSON null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirement_context: Option<Option<RequirementContextDocument>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReplayReportV1Wire {
    schema_version: String,
    status: ReplayStatus,
    expected_report_sha256: String,
    observed_report_sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReplayReportV2Wire {
    schema_version: String,
    status: ReplayStatus,
    expected_report_sha256: String,
    observed_report_sha256: String,
    signal_catalog_sha256: String,
    requirement_context: serde_json::Value,
}

impl<'de> Deserialize<'de> for ReplayReport {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let version = value
            .get("schemaVersion")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| D::Error::custom("replay report requires schemaVersion"))?;
        match version {
            "tl-rewrite.replay/v1" => serde_json::from_value::<ReplayReportV1Wire>(value)
                .map(|wire| Self {
                    schema_version: wire.schema_version,
                    status: wire.status,
                    expected_report_sha256: wire.expected_report_sha256,
                    observed_report_sha256: wire.observed_report_sha256,
                    signal_catalog_sha256: None,
                    requirement_context: None,
                })
                .map_err(D::Error::custom),
            "tl-rewrite.replay/v2" => serde_json::from_value::<ReplayReportV2Wire>(value)
                .and_then(|wire| {
                    let context = if wire.requirement_context.is_null() {
                        None
                    } else {
                        Some(serde_json::from_value(wire.requirement_context)?)
                    };
                    Ok(Self {
                        schema_version: wire.schema_version,
                        status: wire.status,
                        expected_report_sha256: wire.expected_report_sha256,
                        observed_report_sha256: wire.observed_report_sha256,
                        signal_catalog_sha256: Some(wire.signal_catalog_sha256),
                        requirement_context: Some(context),
                    })
                })
                .map_err(D::Error::custom),
            _ => Err(D::Error::custom("unsupported replay report schemaVersion")),
        }
    }
}

/// Re-executes a report's exact identity and options and compares every field.
///
/// Implements: FR-003
pub fn replay(input: &FormulaDocument, expected: &RewriteReport) -> ReplayReport {
    let observed = rewrite(
        input,
        expected.formula_id.clone(),
        expected.options,
        expected.engine_source_revision.clone(),
    );
    let expected_report_sha256 = sha256_json(expected);
    let observed_report_sha256 = sha256_json(&observed);
    ReplayReport {
        schema_version: "tl-rewrite.replay/v1".to_owned(),
        status: if observed == *expected {
            ReplayStatus::Verified
        } else {
            ReplayStatus::Mismatch
        },
        expected_report_sha256,
        observed_report_sha256,
        signal_catalog_sha256: None,
        requirement_context: None,
    }
}

/// Re-executes a contextual request and compares the complete native v2 report.
///
/// Callers must resupply the complete catalog and exact-or-absent context; their
/// values are inputs to the observed request digest rather than copied from the
/// expected record.
///
/// Implements: FR-007-AC-2
pub fn replay_with_context(
    input: &FormulaDocument,
    expected: &RewriteReport,
    signal_catalog: &SignalCatalogDocument,
    requirement_context: Option<RequirementContextDocument>,
) -> ReplayReport {
    let observed = rewrite_with_context(
        input,
        expected.formula_id.clone(),
        expected.options,
        expected.engine_source_revision.clone(),
        signal_catalog,
        requirement_context.clone(),
    );
    let expected_report_sha256 = sha256_json(expected);
    let observed_report_sha256 = sha256_json(&observed);
    ReplayReport {
        schema_version: "tl-rewrite.replay/v2".to_owned(),
        status: if observed == *expected {
            ReplayStatus::Verified
        } else {
            ReplayStatus::Mismatch
        },
        expected_report_sha256,
        observed_report_sha256,
        signal_catalog_sha256: Some(sha256_json(signal_catalog)),
        requirement_context: Some(requirement_context),
    }
}

impl ReplayReport {
    /// Reads one exact canonical replay report under caller-lowerable owner limits.
    ///
    /// Implements: FR-010-AC-5
    pub fn from_json_bytes(bytes: &[u8], limits: RecordLimits) -> Result<Self, RecordReadError> {
        crate::report::read_canonical(bytes, limits)
    }
}
