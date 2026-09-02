use tl_syntax::{
    FormulaDocument, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile,
};

pub fn document(profile: SemanticProfile, nodes: Vec<Node>) -> FormulaDocument {
    FormulaDocument::new(profile, NodeId((nodes.len() - 1) as u32), nodes).unwrap()
}

pub fn proposition(id: u32) -> Node {
    Node::new(NodeKind::Proposition {
        proposition: PropositionId(id),
    })
}

#[allow(dead_code)]
pub fn west_document(id: &str) -> FormulaDocument {
    let interval = Interval::new(0, 2).unwrap();
    let nodes = match id {
        "west-d1-and-idempotent" => vec![
            proposition(1),
            Node::new(NodeKind::Not { operand: NodeId(0) }),
            Node::new(NodeKind::And {
                left: NodeId(1),
                right: NodeId(1),
            }),
        ],
        "west-d1-or-true" => vec![
            proposition(1),
            Node::new(NodeKind::Not { operand: NodeId(0) }),
            Node::new(NodeKind::True),
            Node::new(NodeKind::Or {
                left: NodeId(1),
                right: NodeId(2),
            }),
        ],
        "west-d1-and-true" => vec![
            proposition(0),
            Node::new(NodeKind::Not { operand: NodeId(0) }),
            Node::new(NodeKind::True),
            Node::new(NodeKind::And {
                left: NodeId(1),
                right: NodeId(2),
            }),
        ],
        "west-d1-globally-true" => vec![
            Node::new(NodeKind::True),
            Node::new(NodeKind::Globally {
                interval,
                operand: NodeId(0),
            }),
        ],
        "west-d1-future-true" => vec![
            Node::new(NodeKind::True),
            Node::new(NodeKind::Future {
                interval,
                operand: NodeId(0),
            }),
        ],
        "west-d1-until-true" => vec![
            proposition(1),
            Node::new(NodeKind::Not { operand: NodeId(0) }),
            Node::new(NodeKind::True),
            Node::new(NodeKind::Until {
                interval,
                left: NodeId(2),
                right: NodeId(1),
            }),
        ],
        "west-d1-or-false" => vec![
            Node::new(NodeKind::False),
            proposition(0),
            Node::new(NodeKind::Or {
                left: NodeId(0),
                right: NodeId(1),
            }),
        ],
        "west-d1-and-true-left" => vec![
            Node::new(NodeKind::True),
            proposition(1),
            Node::new(NodeKind::And {
                left: NodeId(0),
                right: NodeId(1),
            }),
        ],
        "west-d1-false-or" => vec![
            Node::new(NodeKind::False),
            Node::new(NodeKind::Or {
                left: NodeId(0),
                right: NodeId(0),
            }),
        ],
        "west-d1-false-and" => vec![
            Node::new(NodeKind::False),
            Node::new(NodeKind::And {
                left: NodeId(0),
                right: NodeId(0),
            }),
        ],
        other => panic!("unknown WEST fixture {other}"),
    };
    document(SemanticProfile::ClosedTraceV1, nodes)
}
