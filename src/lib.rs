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
pub use disposition::{
    conformance_disposition, rewrite_disposition, DispositionMappingError, Fr341Disposition,
};
pub use engine::{rewrite, rewrite_with_context};
pub use equivalence::{
    check_equivalence, check_equivalence_with_context, check_past_equivalence, ConformanceOptions,
    ConformanceReason, ConformanceReport, ConformanceStatus, PastConformanceReason,
    PastConformanceReport, PastEvaluationContext,
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
pub const TL_SYNTAX_REVISION: &str = "4a5614193d21e5ae99950ae683b04ba0ec931358";

/// Exact tl-mltl reference source revision consumed by this candidate.
pub const TL_MLTL_REVISION: &str = "452f013a3168512603d427bce3360bc14c1175a6";

/// Exact canonical WEST source revision from which permitted fixtures were selected.
pub const WEST_REVISION: &str = "21cd99ab2e6095a099dd179029cfdeb54268ad3f";

/// Merged PGM-01 policy revision governing evidence and decision boundaries.
pub const PGM01_POLICY_REVISION: &str = "7dac9d8c19952412b56a0347387666e2ca81e01d";
