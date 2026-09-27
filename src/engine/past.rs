//! Origin-complete past profile admission and rule dispatch.

use tl_syntax::{FormulaDocument, NodeId, NodeKind, SemanticProfile, SourceSpan};

use super::{applied_kind, Abort, PassState};

/// Exact semantic profile owned by the past rewrite engine.
///
/// Implements: FR-010-AC-1
pub const SEMANTIC_PROFILE: SemanticProfile = SemanticProfile::OriginCompleteHistoryV1;

pub(super) fn supports(input: &FormulaDocument) -> bool {
    input.semantic_profile() == SEMANTIC_PROFILE
}

pub(super) fn apply_first(
    state: &mut PassState,
    kind: NodeKind,
    span: Option<SourceSpan>,
) -> Result<Option<(&'static str, u32, NodeId)>, Abort> {
    if let Some(application) = super::boolean::apply_first(state, kind.into(), span)? {
        return Ok(Some(application));
    }
    match kind {
        NodeKind::Not { operand } => match state.kind(operand) {
            Some(NodeKind::Since {
                interval,
                left,
                right,
            }) => match (state.kind(left), state.kind(right)) {
                (
                    Some(NodeKind::Not {
                        operand: inner_left,
                    }),
                    Some(NodeKind::Not {
                        operand: inner_right,
                    }),
                ) => applied_kind(
                    state,
                    "past.triggered.fold-dual",
                    NodeKind::Triggered {
                        interval,
                        left: inner_left,
                        right: inner_right,
                    },
                    span,
                ),
                _ => Ok(None),
            },
            _ => Ok(None),
        },
        NodeKind::Once { interval, operand } if interval.start() == 1 && interval.end() == 1 => {
            applied_kind(
                state,
                "past.once.strong-previous",
                NodeKind::StrongPrevious { operand },
                span,
            )
        }
        _ => Ok(None),
    }
}
