---
id: SR-044
title: "Rust code review of executable ix-flow package scanning"
type: SpecReview
analysis: code-review
scope: "tests/shared_assurance.rs and NFR-003 corrective issue/33 head"
review_set: subset
relationships:
  - target: ix://agent-ix/tl-rewrite/PLAN-005
    type: reviews
  - target: ix://agent-ix/tl-rewrite/TM-001
    type: references
---

## Summary

Reviewed YAML run-script isolation, shell-token classification, npm subcommand
recognition, ix-flow package-family census, fail-closed diagnostics, trigger and
runtime controls, and the sealed requirement projection at `6f13a58`. Two
review-time parser gaps were reproduced, fixed, and retained as TC-039 controls;
no remaining code defect was established.

## Verdict

**PASS** — the Rust assurance implementation agrees with NFR-003-AC-7 through
AC-12 and changes no production rewrite behavior. This authorial review records
local evidence and does not grant independent exact-head clearance.

## Review Evidence

- The scanner extracts inline and literal-block YAML `run` scripts, including
  quoted keys; YAML
  metadata and comments cannot enter the npm package population. Folded or
  otherwise unsupported block-scalar headers fail closed.
- Shell tokens retain literal versus dynamic provenance across unquoted,
  single-quoted, and double-quoted input. GitHub workflow interpolation remains
  dynamic even inside shell quotes; shell/command/process substitution, glob,
  grouping, and redirection shapes are refused as non-literal package arguments.
- Npm `install`, `i`, and `add` are found after global options and their values. The
  exact scoped registry specification is accepted across long and short npm
  spellings; unscoped, unversioned, alias, git/GitHub, URL, tarball/file,
  workspace/link, mixed-case, and duplicate variants are rejected and named.
- Three detached implementation mutations made TC-039 red: metadata intrusion,
  omitted GitHub-shorthand recognition, and disabled non-literal rejection.
- The complete branch-local `make ci` gate passed at `6f13a58`, including 18/18
  shared-assurance tests, 94/94 Quire rows, 106/106 strict documents, exact
  released pins, MSRV, provenance, and the Quoin chain. Hosted CI was not
  dispatched.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-4401 | medium | **FIXED:** the first implementation stopped at the first non-option word after `npm`, so an option value before `install` hid the package population. TC-039 now accepts `npm --prefix /tmp install` and the scanner locates the literal install subcommand. | NFR-003-AC-7, TC-039, `workflow_ix_flow_packages` |
| FND-4402 | medium | **FIXED:** process-substitution punctuation was initially classified as literal. Parentheses and redirection shapes now make package arguments non-literal, and TC-039 refuses `<(printf ix-flow-package)`. | NFR-003-AC-12, TC-039, `shell_tokens` |
| FND-4403 | low | **FIXED:** folded YAML blocks were initially read with literal-block newline semantics, and mixed-case GitHub identities were not normalized. Folded blocks now fail closed and case-insensitive identity recognition is covered. | NFR-003-AC-8, NFR-003-AC-12, TC-039 |
| FND-4404 | low | The control is internal Rust assurance infrastructure and does not add a user-authored TL or Quire language surface. | NFR-003, TC-039, `tests/shared_assurance.rs` |
| FND-4405 | high | **FIXED after independent review of `d8138fa`:** quoted YAML `run` keys were omitted. Run-key recognition now accepts plain, single-quoted, and double-quoted keys, with a retained quoted-key control. | NFR-003-AC-7, TC-039, `workflow_run_scripts` |
| FND-4406 | high | **FIXED after independent review of `d8138fa`:** stripping YAML comments across literal-block content treated every unquoted `#` as non-executable, although shell treats a word-internal hash as argument content. Run scripts are now extracted before shell comments are classified, with positive word-internal and negative word-boundary controls. | NFR-003-AC-9, TC-039, `shell_tokens` |
| FND-4407 | high | **FIXED after independent review of `d8138fa`:** npm `add` is an install alias but was outside the recognized command domain. NFR-003-AC-7/12 and TC-039 now include `add`; the reviewer’s GitHub-spec mutation turns the census red. | NFR-003-AC-7, NFR-003-AC-12, TC-039, `workflow_ix_flow_packages` |
