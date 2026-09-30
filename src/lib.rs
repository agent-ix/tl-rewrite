//! Deterministic, bounded, semantics-preserving MLTL rewriting.
//!
//! This crate consumes validated [`tl_syntax::FormulaDocument`] values. It owns
//! neither text parsing nor a second evaluator: bounded equivalence delegates
//! to the exact pinned `tl-mltl` reference implementation.

#![forbid(unsafe_code)]

pub mod catalog;
pub mod ci_guard;
mod disposition;
pub mod engine;
pub mod equivalence;
mod hash;
pub mod infinite;
pub mod replay;
pub mod report;

pub use catalog::{
    catalog, infinite_catalog, past_catalog, CatalogDocument, Provenance, ProvenanceKind,
    RuleClass, RuleDefinition, RuleDisposition,
};
#[cfg(feature = "infinite-trace")]
pub use disposition::infinite_conformance_disposition;
pub use disposition::{
    conformance_disposition, rewrite_disposition, DispositionMappingError, Fr341Disposition,
};
pub use engine::{rewrite, rewrite_with_context};
pub use equivalence::{
    check_equivalence, check_equivalence_with_context, check_past_equivalence, ConformanceOptions,
    ConformanceReason, ConformanceReport, ConformanceStatus, PastConformanceReason,
    PastConformanceReport, PastEvaluationContext,
};
#[cfg(feature = "infinite-trace")]
pub use infinite::{
    check_infinite_rewrite, InfiniteConformanceReason, InfiniteConformanceReport,
    InfiniteConformanceStatus,
};
pub use infinite::{
    replay_infinite, rewrite_infinite, InfiniteRewriteFailure, InfiniteRewriteReport,
    MAX_INFINITE_REPORT_BYTES,
};
pub use replay::{replay, replay_with_context, ReplayReport, ReplayStatus};
pub use report::{
    BindingFailure, BindingLocus, BudgetKind, RecordLimits, RecordReadError, RecordReadErrorCode,
    RewriteBudgets, RewriteOptions, RewriteReport, RewriteStatus, RewriteStep, RewriteStrategy,
};

/// Exact tl-syntax source revision consumed by this candidate.
pub const TL_SYNTAX_REVISION: &str = "6aa9b11e29040d64b437da87c9944e3dedd34a86";

/// Exact tl-mltl reference source revision consumed by this candidate.
pub const TL_MLTL_REVISION: &str = "1d9a97f6b601bcc5ee7f2b644bf8b5ea3d66e61e";

/// Exact canonical WEST source revision from which permitted fixtures were selected.
pub const WEST_REVISION: &str = "21cd99ab2e6095a099dd179029cfdeb54268ad3f";
