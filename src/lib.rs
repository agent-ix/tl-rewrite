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
pub const TL_SYNTAX_REVISION: &str = "740182f13b84858008d6f176f75136737d405c1b";

/// Exact tl-mltl reference source revision consumed by this candidate.
pub const TL_MLTL_REVISION: &str = "da2c7704a5347d063398c852acf6aa5bf9b5752d";

/// Exact canonical WEST source revision from which permitted fixtures were selected.
pub const WEST_REVISION: &str = "21cd99ab2e6095a099dd179029cfdeb54268ad3f";

/// Merged PGM-01 policy revision governing evidence and decision boundaries.
pub const PGM01_POLICY_REVISION: &str = "7dac9d8c19952412b56a0347387666e2ca81e01d";
