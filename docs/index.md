# tl-rewrite documentation

tl-rewrite rewrites Mission-time Linear Temporal Logic (MLTL) formulas to a normal form with a fixed catalog of semantics-preserving rules, and produces replayable evidence for each rewrite. This directory documents what the crate contains and how to use it. The [README](../README.md) has the overview and the smallest example.

| Page | What it covers |
| --- | --- |
| [how-it-works.md](how-it-works.md) | The rewrite engine, budgets, report statuses, the three profiles, where the crate sits in the pipeline from text to a Quire outcome, and its step in the [debugging walkthrough](https://github.com/agent-ix/tl-mltl/blob/main/docs/scenario.md). |
| [rules.md](rules.md) | The rule catalogs summarized by family, with preconditions, the excluded rules and the rule identifiers. |
| [replay-and-evidence.md](replay-and-evidence.md) | Replay, bounded equivalence, counterexamples, the record readers and the corpora. |
| [examples.md](examples.md) | Runnable examples: budgets, counterexamples, strict report reading, the past catalog. |
| [api-reference.md](api-reference.md) | Public types and functions by module. |
| [DER-001-rule-derivations.md](DER-001-rule-derivations.md) | The rule derivations: the argument for each rule in the future catalog. |

## Evidence

Agreement between a formula and its rewrite is evidence for one formula pair and one bounded domain.
