//! Deterministic, bounded, semantics-preserving MLTL rewriting.
//!
//! This crate consumes validated [`tl_syntax::FormulaDocument`] values. It owns
//! neither text parsing nor a second evaluator: bounded equivalence delegates
//! to the exact pinned `tl-mltl` reference implementation.

#![forbid(unsafe_code)]

pub mod catalog;
pub mod engine;
pub mod equivalence;
mod hash;
pub mod replay;
pub mod report;

pub use catalog::{
    catalog, past_catalog, CatalogDocument, Provenance, ProvenanceKind, RuleClass, RuleDefinition,
    RuleDisposition,
};
pub use engine::{rewrite, rewrite_with_context};
pub use equivalence::{
    check_equivalence, check_equivalence_with_context, check_past_equivalence, ConformanceOptions,
    ConformanceReason, ConformanceReport, ConformanceStatus, PastConformanceReason,
    PastConformanceReport,
};
pub use replay::{replay, replay_with_context, ReplayReport, ReplayStatus};
pub use report::{
    BindingFailure, BindingLocus, BudgetKind, RecordLimits, RecordReadError, RecordReadErrorCode,
    RewriteBudgets, RewriteOptions, RewriteReport, RewriteStatus, RewriteStep, RewriteStrategy,
};

/// Exact tl-syntax source revision consumed by this candidate.
pub const TL_SYNTAX_REVISION: &str = "842d82553f045eb69a7f38745756d968254fc25e";

/// Exact tl-mltl reference source revision consumed by this candidate.
pub const TL_MLTL_REVISION: &str = "22862189ac4eb515ab84928faec25b2eac47d835";

/// Exact canonical WEST source revision from which permitted fixtures were selected.
pub const WEST_REVISION: &str = "21cd99ab2e6095a099dd179029cfdeb54268ad3f";

/// Merged PGM-01 policy revision governing evidence and decision boundaries.
pub const PGM01_POLICY_REVISION: &str = "7dac9d8c19952412b56a0347387666e2ca81e01d";
