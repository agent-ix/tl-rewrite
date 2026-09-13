//! Deterministic, bounded, semantics-preserving MLTL rewriting.
//!
//! This crate consumes validated [`tl_syntax::FormulaDocument`] values. It owns
//! neither text parsing nor a second evaluator: bounded equivalence delegates
//! to the exact pinned `tl-mltl` reference implementation.

#![forbid(unsafe_code)]

mod catalog;
mod equivalence;
mod hash;
mod rewrite;

pub use catalog::{
    catalog, past_catalog, CatalogDocument, Provenance, ProvenanceKind, RuleClass, RuleDefinition,
    RuleDisposition,
};
pub use equivalence::{
    check_equivalence, check_equivalence_with_context, ConformanceOptions, ConformanceReason,
    ConformanceReport, ConformanceStatus,
};
pub use rewrite::{
    replay, replay_with_context, rewrite, rewrite_with_context, BindingFailure, BindingLocus,
    BudgetKind, ReplayReport, ReplayStatus, RewriteBudgets, RewriteOptions, RewriteReport,
    RewriteStatus, RewriteStep, RewriteStrategy,
};

/// Exact tl-syntax source revision consumed by this candidate.
pub const TL_SYNTAX_REVISION: &str = "e70f2379a752117c79603bc399a86c26feed7716";

/// Exact tl-mltl reference source revision consumed by this candidate.
pub const TL_MLTL_REVISION: &str = "b346cd0902794633e862f644a5575fc9776c34fb";

/// Exact canonical WEST source revision from which permitted fixtures were selected.
pub const WEST_REVISION: &str = "21cd99ab2e6095a099dd179029cfdeb54268ad3f";

/// Merged PGM-01 policy revision governing evidence and decision boundaries.
pub const PGM01_POLICY_REVISION: &str = "7dac9d8c19952412b56a0347387666e2ca81e01d";
