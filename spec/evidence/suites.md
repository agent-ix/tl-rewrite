---
id: SUR-001
title: tl-rewrite v0.1 evidence suite registry
type: SuiteRegistry
---

# tl-rewrite v0.1 evidence suite registry

## Suites

| ID | Name | Command | Tool | Evidence Kind |
|---|---|---|---|---|
| SUITE-001 | Complete repository CI | `make ci` | GNU Make and Cargo | Integration |
| SUITE-002 | Specification validation | `make spec` | quire 0.31.0 | Analysis |
| SUITE-003 | Requirement coverage | `quire coverage --scope . --strict` | quire 0.31.0 | Analysis |
| SUITE-004 | Catalog and bounded engine | `cargo test --test rewrite` | Rust test harness | Integration |
| SUITE-005 | Replay and wire conformance | `cargo test --test replay` | Rust test harness | Integration |
| SUITE-006 | tl-mltl and WEST equivalence | `cargo test --test equivalence` | Rust test harness | Integration |
| SUITE-007 | PGM-01 evidence envelope | exact merged schema and validator | Python Draft 7 | Analysis |
