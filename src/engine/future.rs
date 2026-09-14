//! Closed-future profile admission and rule dispatch.

use tl_syntax::{FormulaDocument, NodeId, NodeKind, SemanticProfile, SourceSpan};

use super::{applied, applied_kind, is_false, is_true, Abort, PassState};

/// Exact semantic profile owned by the future rewrite engine.
///
/// Implements: FR-010-AC-1
pub const SEMANTIC_PROFILE: SemanticProfile = SemanticProfile::ClosedTraceV1;

pub(super) fn supports(input: &FormulaDocument) -> bool {
    input.semantic_profile() == SEMANTIC_PROFILE
}

pub(super) fn apply_first(
    state: &mut PassState,
    kind: NodeKind,
    span: Option<SourceSpan>,
) -> Result<Option<(&'static str, u32, NodeId)>, Abort> {
    if let Some(application) = super::boolean::apply_first(state, kind, span)? {
        return Ok(Some(application));
    }
    match kind {
        NodeKind::Not { operand } => match state.kind(operand) {
            Some(NodeKind::Future { interval, operand }) => {
                let negated = state.emit(NodeKind::Not { operand }, None)?;
                let output = state.emit(
                    NodeKind::Globally {
                        interval,
                        operand: negated,
                    },
                    span,
                )?;
                Ok(Some(("neg.future.dual", 1, output)))
            }
            Some(NodeKind::Globally { interval, operand }) => {
                let negated = state.emit(NodeKind::Not { operand }, None)?;
                let output = state.emit(
                    NodeKind::Future {
                        interval,
                        operand: negated,
                    },
                    span,
                )?;
                Ok(Some(("neg.globally.dual", 1, output)))
            }
            Some(NodeKind::Until {
                interval,
                left,
                right,
            }) => {
                let left = state.emit(NodeKind::Not { operand: left }, None)?;
                let right = state.emit(NodeKind::Not { operand: right }, None)?;
                let output = state.emit(
                    NodeKind::Release {
                        interval,
                        left,
                        right,
                    },
                    span,
                )?;
                Ok(Some(("neg.until.dual", 1, output)))
            }
            Some(NodeKind::Release {
                interval,
                left,
                right,
            }) => {
                let left = state.emit(NodeKind::Not { operand: left }, None)?;
                let right = state.emit(NodeKind::Not { operand: right }, None)?;
                let output = state.emit(
                    NodeKind::Until {
                        interval,
                        left,
                        right,
                    },
                    span,
                )?;
                Ok(Some(("neg.release.dual", 1, output)))
            }
            _ => Ok(None),
        },
        NodeKind::Future { interval, operand } if interval.start() == 0 && interval.end() == 0 => {
            applied("temporal.future.singleton", operand)
        }
        NodeKind::Future {
            interval: _,
            operand,
        } if is_false(state, operand) => {
            applied_kind(state, "temporal.future.false", NodeKind::False, span)
        }
        NodeKind::Future {
            interval: _,
            operand,
        } if is_true(state, operand) => {
            applied_kind(state, "temporal.future.true", NodeKind::True, span)
        }
        NodeKind::Globally { interval, operand }
            if interval.start() == 0 && interval.end() == 0 =>
        {
            applied("temporal.globally.singleton", operand)
        }
        NodeKind::Globally {
            interval: _,
            operand,
        } if is_false(state, operand) => {
            applied_kind(state, "temporal.globally.false", NodeKind::False, span)
        }
        NodeKind::Globally {
            interval: _,
            operand,
        } if is_true(state, operand) => {
            applied_kind(state, "temporal.globally.true", NodeKind::True, span)
        }
        NodeKind::Until {
            interval,
            left: _,
            right,
        } if interval.start() == 0 && interval.end() == 0 => {
            applied("temporal.until.singleton", right)
        }
        NodeKind::Until {
            interval,
            left,
            right,
        } if is_true(state, left) => applied_kind(
            state,
            "temporal.until.true-left",
            NodeKind::Future {
                interval,
                operand: right,
            },
            span,
        ),
        NodeKind::Release {
            interval,
            left: _,
            right,
        } if interval.start() == 0 && interval.end() == 0 => {
            applied("temporal.release.singleton", right)
        }
        NodeKind::Release {
            interval,
            left,
            right,
        } if is_false(state, left) => applied_kind(
            state,
            "temporal.release.false-left",
            NodeKind::Globally {
                interval,
                operand: right,
            },
            span,
        ),
        _ => Ok(None),
    }
}
