//! Typed FR-341 correspondence for rewrite and conformance attempts.
//!
//! A rewrite or soundness comparison is not a temporal proof. Only its
//! non-conclusive failure classes can correspond to external result labels.

#[cfg(feature = "infinite-trace")]
use crate::infinite::{InfiniteConformanceReason, InfiniteConformanceStatus};
use crate::{ConformanceReason, ConformanceStatus, RewriteStatus};

/// Exact external result class implied by a rewrite or comparison attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Fr341Disposition {
    /// The attempt has no temporal result label, including a successful rewrite.
    NoTemporalVerdict,
    /// A requested capability or input profile is unavailable.
    Unsupported,
    /// A deterministic work ceiling prevented completion.
    ResourceIncomplete,
    /// An internal evaluator or convergence failure prevented completion.
    Failed,
}

impl Fr341Disposition {
    /// Returns the FR-341 label, if the attempt has one.
    pub const fn label(self) -> Option<&'static str> {
        match self {
            Self::NoTemporalVerdict => None,
            Self::Unsupported => Some("unsupported"),
            Self::ResourceIncomplete | Self::Failed => Some("failed"),
        }
    }

    /// Returns the distinct execution disposition, if the attempt has one.
    pub const fn execution(self) -> Option<&'static str> {
        match self {
            Self::NoTemporalVerdict => None,
            Self::Unsupported => Some("unsupported"),
            Self::ResourceIncomplete => Some("resource-incomplete"),
            Self::Failed => Some("failed"),
        }
    }
}

/// Inconsistent status/reason pair in a conformance report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispositionMappingError {
    /// A non-conclusive status requires a typed reason.
    MissingReason,
    /// A completed comparison cannot carry a non-conclusive reason.
    UnexpectedReason,
}

/// Maps every closed rewrite status without inspecting its message text.
pub const fn rewrite_disposition(status: RewriteStatus) -> Fr341Disposition {
    match status {
        RewriteStatus::Unchanged | RewriteStatus::Normalized => Fr341Disposition::NoTemporalVerdict,
        RewriteStatus::BudgetExhausted => Fr341Disposition::ResourceIncomplete,
        RewriteStatus::NonConvergent | RewriteStatus::Failed => Fr341Disposition::Failed,
        RewriteStatus::InvalidInput
        | RewriteStatus::UnsupportedProfile
        | RewriteStatus::UnresolvedBinding => Fr341Disposition::Unsupported,
    }
}

/// Maps one consistent conformance status/reason pair without promoting a
/// completed equivalence check to a temporal verdict.
pub const fn conformance_disposition(
    status: ConformanceStatus,
    reason: Option<ConformanceReason>,
) -> Result<Fr341Disposition, DispositionMappingError> {
    match (status, reason) {
        (ConformanceStatus::Equivalent | ConformanceStatus::Mismatch, None) => {
            Ok(Fr341Disposition::NoTemporalVerdict)
        }
        (ConformanceStatus::Equivalent | ConformanceStatus::Mismatch, Some(_)) => {
            Err(DispositionMappingError::UnexpectedReason)
        }
        (ConformanceStatus::NonConclusive, None) => Err(DispositionMappingError::MissingReason),
        (ConformanceStatus::NonConclusive, Some(reason)) => Ok(match reason {
            ConformanceReason::InvalidInput
            | ConformanceReason::UnsupportedProfile
            | ConformanceReason::OriginalBinding
            | ConformanceReason::RewrittenBinding => Fr341Disposition::Unsupported,
            ConformanceReason::PropositionLimit
            | ConformanceReason::HorizonLimit
            | ConformanceReason::TraceDomainLimit => Fr341Disposition::ResourceIncomplete,
            ConformanceReason::EvaluatorError => Fr341Disposition::Failed,
        }),
    }
}

/// Maps every infinite conformance state and typed reason without promoting
/// trace-scoped differential evidence to a temporal verdict.
#[cfg(feature = "infinite-trace")]
pub const fn infinite_conformance_disposition(
    status: InfiniteConformanceStatus,
    reason: Option<InfiniteConformanceReason>,
) -> Result<Fr341Disposition, DispositionMappingError> {
    match (status, reason) {
        (InfiniteConformanceStatus::Equivalent | InfiniteConformanceStatus::Mismatch, None) => {
            Ok(Fr341Disposition::NoTemporalVerdict)
        }
        (InfiniteConformanceStatus::Equivalent | InfiniteConformanceStatus::Mismatch, Some(_)) => {
            Err(DispositionMappingError::UnexpectedReason)
        }
        (InfiniteConformanceStatus::NonConclusive, None) => {
            Err(DispositionMappingError::MissingReason)
        }
        (InfiniteConformanceStatus::NonConclusive, Some(reason)) => Ok(match reason {
            InfiniteConformanceReason::InvalidRewrite
            | InfiniteConformanceReason::InvalidTrace
            | InfiniteConformanceReason::ProviderRefusal => Fr341Disposition::Unsupported,
            InfiniteConformanceReason::ConflictingObservation
            | InfiniteConformanceReason::EmptyFairAdmission => Fr341Disposition::NoTemporalVerdict,
            InfiniteConformanceReason::ResourceIncomplete => Fr341Disposition::ResourceIncomplete,
            InfiniteConformanceReason::ProviderFailure => Fr341Disposition::Failed,
        }),
    }
}
