//! FR-021 result-label boundary for rewrite and conformance statuses.

use tl_rewrite::{
    conformance_disposition, rewrite_disposition, ConformanceReason, ConformanceStatus,
    DispositionMappingError, Fr341Disposition, RewriteStatus,
};

// Trace: TC-073, FR-021-AC-1
#[test]
fn tc_073_rewrite_statuses_never_claim_temporal_proof() {
    let cases = [
        (
            RewriteStatus::Unchanged,
            Fr341Disposition::NoTemporalVerdict,
        ),
        (
            RewriteStatus::Normalized,
            Fr341Disposition::NoTemporalVerdict,
        ),
        (
            RewriteStatus::BudgetExhausted,
            Fr341Disposition::ResourceIncomplete,
        ),
        (RewriteStatus::NonConvergent, Fr341Disposition::Failed),
        (RewriteStatus::Failed, Fr341Disposition::Failed),
        (RewriteStatus::InvalidInput, Fr341Disposition::Unsupported),
        (
            RewriteStatus::UnsupportedProfile,
            Fr341Disposition::Unsupported,
        ),
        (
            RewriteStatus::UnresolvedBinding,
            Fr341Disposition::Unsupported,
        ),
    ];
    for (status, expected) in cases {
        let mapped = rewrite_disposition(status);
        assert_eq!(mapped, expected, "status {status:?}");
        assert_ne!(mapped.label(), Some("proved"));
        assert_ne!(mapped.label(), Some("refuted"));
    }
    assert_eq!(
        rewrite_disposition(RewriteStatus::BudgetExhausted).label(),
        Some("failed")
    );
    assert_eq!(
        rewrite_disposition(RewriteStatus::BudgetExhausted).execution(),
        Some("resource-incomplete")
    );
}

// Trace: TC-073, FR-021-AC-1
#[test]
fn tc_073_conformance_reasons_have_exhaustive_typed_mapping() {
    let cases = [
        (
            ConformanceReason::InvalidInput,
            Fr341Disposition::Unsupported,
        ),
        (
            ConformanceReason::UnsupportedProfile,
            Fr341Disposition::Unsupported,
        ),
        (
            ConformanceReason::PropositionLimit,
            Fr341Disposition::ResourceIncomplete,
        ),
        (
            ConformanceReason::HorizonLimit,
            Fr341Disposition::ResourceIncomplete,
        ),
        (
            ConformanceReason::TraceDomainLimit,
            Fr341Disposition::ResourceIncomplete,
        ),
        (ConformanceReason::EvaluatorError, Fr341Disposition::Failed),
        (
            ConformanceReason::OriginalBinding,
            Fr341Disposition::Unsupported,
        ),
        (
            ConformanceReason::RewrittenBinding,
            Fr341Disposition::Unsupported,
        ),
    ];
    for (reason, expected) in cases {
        assert_eq!(
            conformance_disposition(ConformanceStatus::NonConclusive, Some(reason)),
            Ok(expected),
            "reason {reason:?}"
        );
    }
    for status in [ConformanceStatus::Equivalent, ConformanceStatus::Mismatch] {
        assert_eq!(
            conformance_disposition(status, None),
            Ok(Fr341Disposition::NoTemporalVerdict)
        );
        assert_eq!(
            conformance_disposition(status, Some(ConformanceReason::HorizonLimit)),
            Err(DispositionMappingError::UnexpectedReason)
        );
    }
    assert_eq!(
        conformance_disposition(ConformanceStatus::NonConclusive, None),
        Err(DispositionMappingError::MissingReason)
    );
}
