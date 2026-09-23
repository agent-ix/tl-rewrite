use std::{collections::BTreeMap, fs, path::Path};

use sha2::{Digest, Sha256};
use tl_rewrite::{rewrite_infinite, RewriteOptions};
use tl_syntax::InfiniteFormulaDocument;

// Trace: TC-072, FR-020-AC-3, FR-046-AC-1.
#[test]
fn checked_fuzz_seeds_reach_real_infinite_rewrite_rules() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/infinite_rewrite");
    let sums = fs::read_to_string(root.join("SHA256SUMS")).unwrap();
    let pins: BTreeMap<_, _> = sums
        .lines()
        .map(|line| {
            let (hash, name) = line.split_once("  ").expect("hash and filename");
            (name, hash)
        })
        .collect();
    assert_eq!(pins.len(), 3);
    let mut rewritten = 0;
    for (name, expected_hash) in pins {
        let bytes = fs::read(root.join(name)).unwrap();
        assert_eq!(format!("{:x}", Sha256::digest(&bytes)), expected_hash);
        let input: InfiniteFormulaDocument = serde_json::from_slice(&bytes).unwrap();
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
