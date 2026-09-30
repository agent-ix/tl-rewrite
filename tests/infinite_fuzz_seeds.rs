use std::{fs, path::Path};

use tl_rewrite::{rewrite_infinite, RewriteOptions};
use tl_syntax::{InfiniteFormulaDocument, SyntaxArtifactLimits};

// Trace: TC-072, FR-020-AC-3, FR-046-AC-1.
#[test]
fn checked_fuzz_seeds_reach_real_infinite_rewrite_rules() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/infinite_rewrite");
    let mut rewritten = 0;
    for entry in fs::read_dir(&root).unwrap() {
        let path = entry.unwrap().path();
        let name = path.display();
        let bytes = fs::read(&path).unwrap();
        let input =
            InfiniteFormulaDocument::from_json_bytes(&bytes, SyntaxArtifactLimits::default())
                .unwrap();
        let report = rewrite_infinite(
            &input,
            None,
            RewriteOptions::default(),
            "checked-fuzz-seed/v1",
            1_000_000,
        );
        assert!(report.succeeded(), "{name}: {:?}", report.failure);
        rewritten += usize::from(!report.steps.is_empty());
    }
    assert!(rewritten >= 2);
}
