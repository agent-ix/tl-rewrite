//! Deterministic, bounded, semantics-preserving MLTL rewriting.
//!
//! This crate consumes validated [`tl_syntax::FormulaDocument`] values. It owns
//! neither text parsing nor a second evaluator: bounded equivalence delegates
//! to the exact pinned `tl-mltl` reference implementation.

mod catalog;
mod equivalence;
mod hash;
mod rewrite;

pub use catalog::{
    catalog, CatalogDocument, Provenance, ProvenanceKind, RuleClass, RuleDefinition,
    RuleDisposition,
};
pub use equivalence::{
    check_equivalence, ConformanceOptions, ConformanceReason, ConformanceReport, ConformanceStatus,
};
pub use rewrite::{
    replay, rewrite, BudgetKind, ReplayReport, ReplayStatus, RewriteBudgets, RewriteOptions,
    RewriteReport, RewriteStatus, RewriteStep, RewriteStrategy,
};

/// Exact tl-syntax source revision consumed by this candidate.
pub const TL_SYNTAX_REVISION: &str = "5e59a26d71b4b5d79623850cda50010e18a90dad";

/// Exact tl-mltl reference source revision consumed by this candidate.
pub const TL_MLTL_REVISION: &str = "a9b7847199c1d846abd7b67901cd6836374ccee2";

/// Exact canonical WEST source revision from which permitted fixtures were selected.
pub const WEST_REVISION: &str = "21cd99ab2e6095a099dd179029cfdeb54268ad3f";

/// Merged PGM-01 policy revision governing evidence and decision boundaries.
pub const PGM01_POLICY_REVISION: &str = "7dac9d8c19952412b56a0347387666e2ca81e01d";
