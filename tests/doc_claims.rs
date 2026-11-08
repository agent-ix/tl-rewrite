//! Documentation and evidence text claims (FR-005-AC-3).
//!
//! Reads the README, `docs/`, and the limitation strings the crate emits,
//! and checks that human authority and the bounded-equivalence limits are
//! stated and that no automated release or qualification claim is made
//! except inside an explicit negation.

use std::{fs, path::Path};

const CLAIM_WORDS: [&str; 6] = [
    "release decision",
    "release approv",
    "release ready",
    "qualif",
    "certif",
    "accredit",
];
const NEGATIONS: [&str; 8] = [
    "no ",
    "not ",
    "never",
    "without",
    "cannot",
    "does not",
    "makes no",
    "remains the responsibility",
];

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect(path)
}

fn corpus() -> Vec<(String, String)> {
    let mut files = vec!["README.md".to_owned()];
    let mut docs: Vec<_> = fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("docs"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        .map(|e| format!("docs/{}", e.file_name().to_string_lossy()))
        .collect();
    docs.sort();
    files.extend(docs);
    // Evidence text the crate emits lives in these sources.
    files.push("src/equivalence.rs".to_owned());
    files
        .into_iter()
        .map(|f| {
            let t = read(&f);
            (f, t)
        })
        .collect()
}

/// Sentences that mention a claim word without an explicit negation.
fn unnegated_claims(text: &str) -> Vec<String> {
    text.split(['.', '\n', ';'])
        .map(|s| s.trim().to_lowercase())
        .filter(|s| CLAIM_WORDS.iter().any(|w| s.contains(w)))
        .filter(|s| !NEGATIONS.iter().any(|n| s.contains(n)))
        .collect()
}

// Trace: FR-005-AC-3, TC-905
#[test]
fn docs_and_evidence_state_human_authority_and_bounded_limits_without_automated_claims() {
    let corpus = corpus();
    let all: String = corpus
        .iter()
        .map(|(_, t)| t.to_lowercase())
        .collect::<Vec<_>>()
        .join("\n");

    let index = read("docs/index.md").to_lowercase();
    assert!(
        index.contains("human reviewers keep authority"),
        "docs/index.md must state human authority"
    );
    assert!(
        index.contains("does not make any release"),
        "docs/index.md must disclaim automated decisions"
    );
    assert!(
        all.contains("does not prove"),
        "bounded-equivalence limitation missing"
    );
    let equivalence = read("src/equivalence.rs");
    assert!(
        equivalence.matches("does not prove").count() >= 2
            && equivalence.contains("qualify a consuming tool"),
        "emitted limitation strings must restate the bounded limit and disclaim qualification"
    );

    for (file, text) in &corpus {
        // In source, only the emitted limitation strings are evidence text.
        let scanned: String = if file.starts_with("src/") {
            text.lines()
                .filter(|l| l.contains("limitation"))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            text.clone()
        };
        let bad = unnegated_claims(&scanned);
        assert!(
            bad.is_empty(),
            "{file}: unnegated release/qualification claim: {bad:?}"
        );
    }
}
