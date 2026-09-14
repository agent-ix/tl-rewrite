//! Profile-independent Boolean algebra shared by admitted rewrite profiles.

use tl_syntax::{NodeId, NodeKind, SourceSpan};

use super::{applied, applied_kind, is_false, is_true, Abort, PassState};

fn apply_boolean_not(
    state: &mut PassState,
    operand: NodeId,
    span: Option<SourceSpan>,
) -> Result<Option<(&'static str, u32, NodeId)>, Abort> {
    match state.kind(operand) {
        Some(NodeKind::False) => applied_kind(state, "bool.not.false", NodeKind::True, span),
        Some(NodeKind::True) => applied_kind(state, "bool.not.true", NodeKind::False, span),
        Some(NodeKind::Not { operand }) => applied("bool.not.double", operand),
        _ => Ok(None),
    }
}

pub(super) fn apply_first(
    state: &mut PassState,
    kind: NodeKind,
    span: Option<SourceSpan>,
) -> Result<Option<(&'static str, u32, NodeId)>, Abort> {
    match kind {
        NodeKind::Not { operand } => apply_boolean_not(state, operand, span),
        NodeKind::And { left, right: _ } if is_false(state, left) => {
            applied_kind(state, "bool.and.false-left", NodeKind::False, span)
        }
        NodeKind::And { left: _, right } if is_false(state, right) => {
            applied_kind(state, "bool.and.false-right", NodeKind::False, span)
        }
        NodeKind::And { left, right } if is_true(state, left) => {
            applied("bool.and.true-left", right)
        }
        NodeKind::And { left, right } if is_true(state, right) => {
            applied("bool.and.true-right", left)
        }
        NodeKind::And { left, right } if left == right => applied("bool.and.idempotent", left),
        NodeKind::Or { left, right: _ } if is_true(state, left) => {
            applied_kind(state, "bool.or.true-left", NodeKind::True, span)
        }
        NodeKind::Or { left: _, right } if is_true(state, right) => {
            applied_kind(state, "bool.or.true-right", NodeKind::True, span)
        }
        NodeKind::Or { left, right } if is_false(state, left) => {
            applied("bool.or.false-left", right)
        }
        NodeKind::Or { left, right } if is_false(state, right) => {
            applied("bool.or.false-right", left)
        }
        NodeKind::Or { left, right } if left == right => applied("bool.or.idempotent", left),
        NodeKind::Implies { left, right: _ } if is_false(state, left) => {
            applied_kind(state, "bool.implies.false-left", NodeKind::True, span)
        }
        NodeKind::Implies { left, right } if is_true(state, left) => {
            applied("bool.implies.true-left", right)
        }
        NodeKind::Implies { left: _, right } if is_true(state, right) => {
            applied_kind(state, "bool.implies.true-right", NodeKind::True, span)
        }
        NodeKind::Implies { left, right } if is_false(state, right) => applied_kind(
            state,
            "bool.implies.false-right",
            NodeKind::Not { operand: left },
            span,
        ),
        NodeKind::Implies { left, right } if left == right => {
            applied_kind(state, "bool.implies.reflexive", NodeKind::True, span)
        }
        NodeKind::Implies { left, right } => {
            let negated = state.emit(NodeKind::Not { operand: left }, None)?;
            let output = state.emit(
                NodeKind::Or {
                    left: negated,
                    right,
                },
                span,
            )?;
            Ok(Some(("bool.implies.eliminate", 1, output)))
        }
        NodeKind::Equivalent { left, right } if left == right => {
            applied_kind(state, "bool.equivalent.reflexive", NodeKind::True, span)
        }
        NodeKind::Equivalent { left, right } if is_true(state, left) => {
            applied("bool.equivalent.true-left", right)
        }
        NodeKind::Equivalent { left, right } if is_true(state, right) => {
            applied("bool.equivalent.true-right", left)
        }
        NodeKind::Equivalent { left, right } if is_false(state, left) => applied_kind(
            state,
            "bool.equivalent.false-left",
            NodeKind::Not { operand: right },
            span,
        ),
        NodeKind::Equivalent { left, right } if is_false(state, right) => applied_kind(
            state,
            "bool.equivalent.false-right",
            NodeKind::Not { operand: left },
            span,
        ),
        _ => Ok(None),
    }
}
