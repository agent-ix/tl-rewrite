//! Profile-independent Boolean algebra shared by admitted rewrite profiles.

use tl_syntax::{InfiniteNodeKind, NodeId, NodeKind, SourceSpan};

/// Boolean view of either owner graph edition. Temporal nodes are opaque here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BoolKind {
    False,
    True,
    Not { operand: NodeId },
    And { left: NodeId, right: NodeId },
    Or { left: NodeId, right: NodeId },
    Implies { left: NodeId, right: NodeId },
    Equivalent { left: NodeId, right: NodeId },
    Other,
}

impl From<NodeKind> for BoolKind {
    fn from(kind: NodeKind) -> Self {
        match kind {
            NodeKind::False => Self::False,
            NodeKind::True => Self::True,
            NodeKind::Not { operand } => Self::Not { operand },
            NodeKind::And { left, right } => Self::And { left, right },
            NodeKind::Or { left, right } => Self::Or { left, right },
            NodeKind::Implies { left, right } => Self::Implies { left, right },
            NodeKind::Equivalent { left, right } => Self::Equivalent { left, right },
            _ => Self::Other,
        }
    }
}

impl From<InfiniteNodeKind> for BoolKind {
    fn from(kind: InfiniteNodeKind) -> Self {
        match kind {
            InfiniteNodeKind::False => Self::False,
            InfiniteNodeKind::True => Self::True,
            InfiniteNodeKind::Not { operand } => Self::Not { operand },
            InfiniteNodeKind::And { left, right } => Self::And { left, right },
            InfiniteNodeKind::Or { left, right } => Self::Or { left, right },
            InfiniteNodeKind::Implies { left, right } => Self::Implies { left, right },
            InfiniteNodeKind::Equivalent { left, right } => Self::Equivalent { left, right },
            _ => Self::Other,
        }
    }
}

/// Only graph storage and failure accounting vary across the two editions.
pub(crate) trait BoolState {
    type Error;

    fn kind(&self, id: NodeId) -> Option<BoolKind>;
    fn emit(&mut self, kind: BoolKind, span: Option<SourceSpan>) -> Result<NodeId, Self::Error>;
    fn existing_constant(
        &mut self,
        id: NodeId,
        kind: BoolKind,
        span: Option<SourceSpan>,
    ) -> Result<NodeId, Self::Error>;
}

impl BoolState for super::PassState {
    type Error = super::Abort;

    fn kind(&self, id: NodeId) -> Option<BoolKind> {
        super::PassState::kind(self, id).map(Into::into)
    }

    fn emit(&mut self, kind: BoolKind, span: Option<SourceSpan>) -> Result<NodeId, Self::Error> {
        let kind = match kind {
            BoolKind::False => NodeKind::False,
            BoolKind::True => NodeKind::True,
            BoolKind::Not { operand } => NodeKind::Not { operand },
            BoolKind::Or { left, right } => NodeKind::Or { left, right },
            _ => unreachable!("Boolean rules emit only constants, negation, and disjunction"),
        };
        super::PassState::emit(self, kind, span)
    }

    fn existing_constant(
        &mut self,
        _id: NodeId,
        kind: BoolKind,
        span: Option<SourceSpan>,
    ) -> Result<NodeId, Self::Error> {
        BoolState::emit(self, kind, span)
    }
}

type Application = (&'static str, u32, NodeId);

fn applied<E>(id: &'static str, output: NodeId) -> Result<Option<Application>, E> {
    Ok(Some((id, 1, output)))
}

fn emitted<S: BoolState>(
    state: &mut S,
    id: &'static str,
    kind: BoolKind,
    span: Option<SourceSpan>,
) -> Result<Option<Application>, S::Error> {
    applied(id, state.emit(kind, span)?)
}

fn retained<S: BoolState>(
    state: &mut S,
    id: &'static str,
    node: NodeId,
    kind: BoolKind,
    span: Option<SourceSpan>,
) -> Result<Option<Application>, S::Error> {
    applied(id, state.existing_constant(node, kind, span)?)
}

fn is_false<S: BoolState>(state: &S, id: NodeId) -> bool {
    state.kind(id) == Some(BoolKind::False)
}

fn is_true<S: BoolState>(state: &S, id: NodeId) -> bool {
    state.kind(id) == Some(BoolKind::True)
}

/// Selects one Boolean rule for either validated graph edition.
pub(crate) fn apply_first<S: BoolState>(
    state: &mut S,
    kind: BoolKind,
    span: Option<SourceSpan>,
) -> Result<Option<Application>, S::Error> {
    match kind {
        BoolKind::Not { operand } => match state.kind(operand) {
            Some(BoolKind::False) => emitted(state, "bool.not.false", BoolKind::True, span),
            Some(BoolKind::True) => emitted(state, "bool.not.true", BoolKind::False, span),
            Some(BoolKind::Not { operand }) => applied("bool.not.double", operand),
            _ => Ok(None),
        },
        BoolKind::And { left, right: _ } if is_false(state, left) => {
            retained(state, "bool.and.false-left", left, BoolKind::False, span)
        }
        BoolKind::And { left: _, right } if is_false(state, right) => {
            retained(state, "bool.and.false-right", right, BoolKind::False, span)
        }
        BoolKind::And { left, right } if is_true(state, left) => {
            applied("bool.and.true-left", right)
        }
        BoolKind::And { left, right } if is_true(state, right) => {
            applied("bool.and.true-right", left)
        }
        BoolKind::And { left, right } if left == right => applied("bool.and.idempotent", left),
        BoolKind::Or { left, right: _ } if is_true(state, left) => {
            retained(state, "bool.or.true-left", left, BoolKind::True, span)
        }
        BoolKind::Or { left: _, right } if is_true(state, right) => {
            retained(state, "bool.or.true-right", right, BoolKind::True, span)
        }
        BoolKind::Or { left, right } if is_false(state, left) => {
            applied("bool.or.false-left", right)
        }
        BoolKind::Or { left, right } if is_false(state, right) => {
            applied("bool.or.false-right", left)
        }
        BoolKind::Or { left, right } if left == right => applied("bool.or.idempotent", left),
        BoolKind::Implies { left, right: _ } if is_false(state, left) => {
            emitted(state, "bool.implies.false-left", BoolKind::True, span)
        }
        BoolKind::Implies { left, right } if is_true(state, left) => {
            applied("bool.implies.true-left", right)
        }
        BoolKind::Implies { left: _, right } if is_true(state, right) => retained(
            state,
            "bool.implies.true-right",
            right,
            BoolKind::True,
            span,
        ),
        BoolKind::Implies { left, right } if is_false(state, right) => emitted(
            state,
            "bool.implies.false-right",
            BoolKind::Not { operand: left },
            span,
        ),
        BoolKind::Implies { left, right } if left == right => {
            emitted(state, "bool.implies.reflexive", BoolKind::True, span)
        }
        BoolKind::Implies { left, right } => {
            let not_left = state.emit(BoolKind::Not { operand: left }, None)?;
            emitted(
                state,
                "bool.implies.eliminate",
                BoolKind::Or {
                    left: not_left,
                    right,
                },
                span,
            )
        }
        BoolKind::Equivalent { left, right } if left == right => {
            emitted(state, "bool.equivalent.reflexive", BoolKind::True, span)
        }
        BoolKind::Equivalent { left, right } if is_true(state, left) => {
            applied("bool.equivalent.true-left", right)
        }
        BoolKind::Equivalent { left, right } if is_true(state, right) => {
            applied("bool.equivalent.true-right", left)
        }
        BoolKind::Equivalent { left, right } if is_false(state, left) => emitted(
            state,
            "bool.equivalent.false-left",
            BoolKind::Not { operand: right },
            span,
        ),
        BoolKind::Equivalent { left, right } if is_false(state, right) => emitted(
            state,
            "bool.equivalent.false-right",
            BoolKind::Not { operand: left },
            span,
        ),
        _ => Ok(None),
    }
}
