---
id: SR-092
title: "Code review of TL-235 and TL-231 rewrite qualification port"
type: SpecReview
analysis: code-review
scope: "agent-ix/tl-rewrite@7e9d091beb0216770d5150b4f49215500caf1426; Cargo.toml, Cargo.lock, benches/README.md, benches/input-digests.json, benches/rewrite_rules.rs, tests/infinite_owner_corpus.rs, tests/shared_assurance.rs, spec/requirements/FR-019-infinite-rule-applicability.md, spec/infinite-rewrite-test-matrix.md, spec/test-matrix.md"
review_set: subset
---

## Summary

Ticket TL-235 with TL-231 owner-corpus adjunct; draft PR #54 against main. Reviewed the full changed path set, public rewrite benchmark, owner corpus replay, pinned dependencies, and CI workflow diff (none). Focused fmt, strict Clippy, owner-corpus test, benchmark test, cargo deny, and Quire validation passed.

## Assurance Context

AP-001 applies to exact source, dependency, and conformance-corpus identity. Candidate 7e9d091 was compared with origin/main. The profile's semantic-drift and false-completion impacts motivate the negative owner-case review. The pinned 0.3 syntax, parse, and mlTL dependencies and corpus were available. No architecture or measurement-plan artifact for this adjunct, independent V9 paired distributions, current Campaign receipt, or producer-reliance decision was available in this review; the PR does not claim those gates. AP-001 declares no implicit exception.

## Verdict

CONDITIONAL. Two medium findings need a fix round before this focused PR can supply trustworthy benchmark and refusal evidence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Criterion times assertion and report inspection in every rewrite sample, obscuring the rewrite-only small-case measurement. | benches/rewrite_rules.rs:27-43,67-69 |
| FND-002 | medium | Clock and fairness negative owner cases bypass admission APIs and count raw-field checks as refusals. | tests/infinite_owner_corpus.rs:116-125 |

## Coverage

The three canonical benchmark digests and public outcomes pass preflight. The 15-case owner corpus test passes, including the finite-prefix call to `evaluate_prefix_safety`; 11 cases are compared and four counted as refusals. Quire reports 166/166 backed matrix rows. Full TL-235 paired Campaign measurement and TL-231 mutation thresholds remain outside this PR's claimed completion.

## Dispositions

Round 1 reviewed `45011e2c6e5fb1b16f0c99cb7a6c70cd064dc912`. The benchmark closure now times only the public rewrite report production, with outcome verification before and after Criterion's samples. The clock and fairness negatives each invoke the syntax owner's selected-identity constructor and assert its typed `Clock` or `EmptyLoop` error. Focused fmt, strict Clippy, owner-corpus test, and bench `--test` pass.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 45011e2c6e5fb1b16f0c99cb7a6c70cd064dc912 |
| FND-002 | fixed | 45011e2c6e5fb1b16f0c99cb7a6c70cd064dc912 |
