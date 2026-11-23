//! The `solver` feature boundary (FR-091-AC-1, FR-091-AC-5, NFR-076).
//!
//! Compiled in every configuration. The manifest checks are structural reads
//! of `Cargo.toml`, `deny.toml` and the `Makefile`; no recorded tree baseline
//! is kept (NFR-076 Verification). The graph checks ask `cargo tree` for the
//! exact configuration they name, independent of the features this test
//! binary was built with.
//!
//! UNBUILT: written without any cargo run (TL-DECISIONS-2026-11-23).

use std::{collections::BTreeSet, fs, path::Path, process::Command};

use tl_rewrite::{check_equivalence, ci_guard::parse_prerequisites, ConformanceOptions};
use tl_syntax::{FormulaDocument, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

const SOLVER_PACKAGES: [&str; 4] = ["tl-solve", "rustsat", "rustsat-batsat", "batsat"];

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect(path)
}

/// The body of one `[section]` of a TOML-like file, up to the next header.
fn section(text: &str, header: &str) -> String {
    let mut inside = false;
    let mut body = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if inside {
                break;
            }
            inside = trimmed == header;
            continue;
        }
        if inside {
            body.push(line);
        }
    }
    body.join("\n")
}

/// `name = ...` entries of a dependency section, without comment lines.
fn entries(body: &str) -> Vec<(String, String)> {
    body.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| {
            let (name, value) = line.split_once('=')?;
            Some((name.trim().to_owned(), value.trim().to_owned()))
        })
        .collect()
}

// Trace: TC-980, NFR-076-AC-1
#[test]
fn manifest_keeps_the_default_dependency_set_and_gates_tl_solve_behind_solver() {
    let manifest = read("Cargo.toml");
    let features = section(&manifest, "[features]");
    let features = entries(&features);
    let feature = |name: &str| {
        features
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())
    };
    assert_eq!(feature("default").as_deref(), Some("[]"));
    assert_eq!(
        feature("solver").as_deref(),
        Some(r#"["dep:tl-solve", "tl-solve/inprocess-sat"]"#)
    );
    assert_eq!(
        feature("infinite-trace").as_deref(),
        Some(r#"["tl-mltl/infinite-trace"]"#)
    );

    let dependencies = entries(&section(&manifest, "[dependencies]"));
    // The default normal set, recorded here rather than as a tree baseline:
    // exactly the five non-optional dependencies tl-rewrite had before FR-091.
    let default_set: BTreeSet<&str> = dependencies
        .iter()
        .filter(|(_, value)| !value.contains("optional = true"))
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(
        default_set,
        BTreeSet::from(["serde", "serde_json", "sha2", "tl-mltl", "tl-syntax"])
    );
    let optional: Vec<&(String, String)> = dependencies
        .iter()
        .filter(|(_, value)| value.contains("optional = true"))
        .collect();
    assert_eq!(optional.len(), 1, "tl-solve is the only optional dependency");
    let (name, value) = optional[0];
    assert_eq!(name, "tl-solve");
    assert!(value.contains("default-features = false"));
    assert!(value.contains(r#"version = "=0.1.0""#));
    assert!(value.contains("rev = "));
    assert!(!value.contains("features = ["), "features come only from `solver`");

    assert!(section(&manifest, "[build-dependencies]").trim().is_empty());
    for (name, _) in entries(&section(&manifest, "[dev-dependencies]")) {
        assert!(
            !SOLVER_PACKAGES.contains(&name.as_str()),
            "{name} would enter the default test build"
        );
    }
    for package in ["rustsat", "batsat"] {
        assert!(
            !dependencies.iter().any(|(name, _)| name.contains(package)),
            "{package} is reached only through tl-solve"
        );
    }
}

fn cargo_tree(prefix: &str, arguments: &[&str]) -> String {
    let output = Command::new(env!("CARGO"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["tree", "--offline", "--locked", "--prefix", prefix])
        .args(arguments)
        .output()
        .expect("cargo tree runs");
    assert!(
        output.status.success(),
        "cargo tree {arguments:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

/// Distinct package lines (`name vX.Y.Z (source)`), with the `(*)`
/// de-duplication marker removed.
fn packages(tree: &str) -> BTreeSet<String> {
    tree.lines()
        .map(|line| line.trim().trim_end_matches("(*)").trim_end().to_owned())
        .filter(|line| !line.is_empty())
        .collect()
}

fn package_name(line: &str) -> &str {
    line.split(' ').next().unwrap_or("")
}

/// Normal and build package lines of one configuration.
fn tree(features: &[&str]) -> BTreeSet<String> {
    let mut arguments = vec!["-e", "normal,build"];
    for feature in features {
        arguments.extend(["--features", *feature]);
    }
    packages(&cargo_tree("none", &arguments))
}

/// The package lines at and below the first `root` entry of a
/// `--prefix depth` tree (tl-solve and its own dependencies, as resolved in
/// that graph). `--features` cannot select a non-member's subtree, so the
/// subtree is cut out of the root package's tree.
fn subtree(features: &[&str], root: &str) -> BTreeSet<String> {
    let mut arguments = vec!["-e", "normal,build"];
    for feature in features {
        arguments.extend(["--features", *feature]);
    }
    let output = cargo_tree("depth", &arguments);
    let mut lines = BTreeSet::new();
    let mut root_depth: Option<usize> = None;
    for line in output.lines() {
        let digits: String = line.chars().take_while(char::is_ascii_digit).collect();
        let Ok(depth) = digits.parse::<usize>() else {
            continue;
        };
        let package = line[digits.len()..].trim().trim_end_matches("(*)").trim_end();
        match root_depth {
            None if package_name(package) == root => {
                root_depth = Some(depth);
                lines.insert(package.to_owned());
            }
            None => {}
            Some(base) if depth > base => {
                lines.insert(package.to_owned());
            }
            Some(_) => break,
        }
    }
    assert!(root_depth.is_some(), "{root} is in the {features:?} tree");
    lines
}

// Trace: TC-980, NFR-076-AC-1
#[test]
fn default_tree_has_no_solver_package_and_the_solver_tree_adds_only_tl_solve_and_its_dependencies()
{
    let default_tree = tree(&[]);
    for line in &default_tree {
        assert!(
            !SOLVER_PACKAGES.contains(&package_name(line)),
            "default tree holds {line}"
        );
    }
    let solver_tree = tree(&["solver"]);
    let tl_solve_tree = subtree(&["solver"], "tl-solve");
    assert!(default_tree.is_subset(&solver_tree));
    for line in solver_tree.difference(&default_tree) {
        assert!(
            tl_solve_tree.contains(line),
            "{line} is in the solver tree but is not tl-solve or one of its dependencies"
        );
    }
}

// Trace: TC-979, FR-091-AC-5
#[test]
fn solver_graph_has_one_tl_syntax_one_tl_mltl_no_infinite_activation_and_no_cycle() {
    let solver_tree = tree(&["solver"]);
    for name in ["tl-syntax", "tl-mltl", "tl-solve"] {
        let count = solver_tree
            .iter()
            .filter(|line| package_name(line) == name)
            .count();
        assert_eq!(count, 1, "{name} must resolve to exactly one package");
    }

    let features = |requested: &str| {
        cargo_tree(
            "none",
            &["-e", "features", "--features", requested, "-i", "tl-mltl"],
        )
    };
    assert!(!features("solver").contains("infinite-trace"));
    assert!(features("solver,infinite-trace").contains("infinite-trace"));

    assert!(
        !subtree(&["solver"], "tl-solve")
            .iter()
            .any(|line| package_name(line) == "tl-rewrite"),
        "tl-solve must not depend on tl-rewrite"
    );
}

// Trace: TC-981, NFR-076-AC-2
#[test]
fn deny_admits_only_tl_solve_and_ci_declares_the_solver_gate() {
    let deny = read("deny.toml");
    let allow_git: BTreeSet<&str> = deny
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("\"https://"))
        .map(|line| line.trim_end_matches(',').trim_matches('"'))
        .collect();
    assert_eq!(
        allow_git,
        BTreeSet::from([
            "https://github.com/agent-ix/tl-mltl.git",
            "https://github.com/agent-ix/tl-oracle.git",
            "https://github.com/agent-ix/tl-parse.git",
            "https://github.com/agent-ix/tl-solve.git",
            "https://github.com/agent-ix/tl-syntax.git",
        ])
    );

    let makefile = read("Makefile");
    let declared = parse_prerequisites(&makefile, "ci").expect("ci declares prerequisites");
    assert!(declared.contains("test-solver"));
    assert!(declared.contains("test-features"));
    let recipe: Vec<&str> = makefile
        .lines()
        .skip_while(|line| !line.starts_with("test-solver:"))
        .skip(1)
        .take_while(|line| line.starts_with('\t'))
        .collect();
    assert!(recipe.iter().any(|line| line.contains("--features solver")));
    assert_eq!(
        recipe.last().map(|line| line.trim()),
        Some("$(CI_GUARD) record test-solver")
    );
}

/// The field set of `tl-rewrite.conformance/v1`, recorded before FR-091.
const CONFORMANCE_V1_FIELDS: [&str; 20] = [
    "catalogSha256",
    "catalogVersion",
    "comparisonId",
    "counterexample",
    "horizon",
    "limitation",
    "options",
    "originalSha256",
    "originalVerdict",
    "propositionIds",
    "reason",
    "rewrittenSha256",
    "rewrittenVerdict",
    "schemaVersion",
    "semanticProfile",
    "status",
    "totalTraces",
    "traceLength",
    "tracesChecked",
    "westRevision",
];

// Trace: TC-975, FR-091-AC-1
#[test]
fn the_conformance_v1_record_is_the_same_in_every_configuration() {
    // SPEC-GAP: FR-091-AC-1 compares conformance/v1 outputs with "recorded
    // values", but no byte-level recordings exist in this repository. The
    // field set is recorded here; `make test-features` and `make test-solver`
    // run this same test in both configurations, so a solver-only field or
    // value change fails one of them.
    let nodes = vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::Not { operand: NodeId(0) }),
        Node::new(NodeKind::Not { operand: NodeId(1) }),
    ];
    let original =
        FormulaDocument::new(SemanticProfile::ClosedTraceV1, NodeId(2), nodes.clone()).unwrap();
    let rewritten =
        FormulaDocument::new(SemanticProfile::ClosedTraceV1, NodeId(0), nodes[..1].to_vec())
            .unwrap();
    let report = check_equivalence(&original, &rewritten, "tc-975", ConformanceOptions::default());
    let value = serde_json::to_value(&report).unwrap();
    let fields: BTreeSet<&str> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(fields, BTreeSet::from(CONFORMANCE_V1_FIELDS));
    assert_eq!(value["schemaVersion"], "tl-rewrite.conformance/v1");
    assert_eq!(value["status"], "equivalent");
    assert_eq!(value["totalTraces"], 2);
    assert_eq!(value["tracesChecked"], 2);
}

// Trace: TC-975, FR-091-AC-1
#[test]
fn every_solver_item_in_the_crate_root_is_behind_the_solver_feature() {
    let lib = read("src/lib.rs");
    let lines: Vec<&str> = lib.lines().collect();
    let mut seen = 0;
    for (index, line) in lines.iter().enumerate() {
        let is_item = line.starts_with("pub mod solver_equivalence")
            || line.starts_with("pub use solver_equivalence");
        if is_item {
            seen += 1;
            assert_eq!(
                index.checked_sub(1).map(|previous| lines[previous].trim()),
                Some("#[cfg(feature = \"solver\")]"),
                "line {} is not feature-gated",
                index + 1
            );
        }
    }
    assert_eq!(seen, 2);
    // No other source file names the module or tl-solve.
    for entry in fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("src")).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if path.extension().is_some_and(|ext| ext == "rs")
            && name != "lib.rs"
            && name != "solver_equivalence.rs"
        {
            let text = fs::read_to_string(&path).unwrap();
            assert!(!text.contains("tl_solve"), "{name} names tl_solve");
        }
    }
}
