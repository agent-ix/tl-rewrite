use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use tl_rewrite::{rewrite, RewriteBudgets, RewriteOptions, RewriteReport, RewriteStatus};
use tl_syntax::{FormulaDocument, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

fn input(applications: u32) -> FormulaDocument {
    let mut nodes = vec![Node::new(NodeKind::Proposition {
        proposition: PropositionId(0),
    })];
    let mut root = NodeId(0);
    for _ in 0..applications {
        let truth = NodeId(u32::try_from(nodes.len()).unwrap());
        nodes.push(Node::new(NodeKind::True));
        root = NodeId(u32::try_from(nodes.len()).unwrap());
        nodes.push(Node::new(NodeKind::And {
            left: truth,
            right: NodeId(root.0 - 2),
        }));
    }
    FormulaDocument::new(SemanticProfile::ClosedTraceV1, root, nodes).unwrap()
}

fn run(document: &FormulaDocument, applications: u32) -> RewriteReport {
    rewrite(
        document,
        "v9-rules",
        RewriteOptions {
            budgets: RewriteBudgets {
                max_nodes: applications * 2 + 1,
                max_rule_applications: u64::from(applications),
                ..RewriteBudgets::default()
            },
            ..RewriteOptions::default()
        },
        "v9-source",
    )
}

fn verify(report: &RewriteReport, applications: u32) {
    assert_eq!(report.status, RewriteStatus::Normalized);
    assert_eq!(report.steps.len(), applications as usize);
    assert_eq!(report.output.as_ref().unwrap().nodes().len(), 1);
}

fn rule_application(c: &mut Criterion) {
    let mut group = c.benchmark_group("rewrite_rules");
    group.sample_size(20);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(1));
    for (name, applications) in [("small_1", 1), ("median_24", 24), ("near_cap_64", 64)] {
        let document = input(applications);
        verify(&run(&document, applications), applications);
        group.throughput(Throughput::Elements(u64::from(applications)));
        group.bench_function(name, |b| {
            b.iter(|| black_box(run(black_box(&document), applications)));
        });
        verify(&run(&document, applications), applications);
    }
    group.finish();
}

criterion_group!(benches, rule_application);
criterion_main!(benches);
