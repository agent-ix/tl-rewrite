# How it works

## The rewrite engine

`rewrite(input, formula_id, options, source_revision)` takes a `FormulaDocument` from tl-syntax. The document must pass `validate()` and carry the closed-trace or the origin-complete-history profile. A document with any other profile gets the status `UnsupportedProfile`, and an invalid document gets `InvalidInput`. Neither produces an output.

The engine works in passes. In each pass it visits the nodes bottom-up, rebuilds each node from its already-rewritten operands and applies the first rule in the catalog whose precondition matches. Rule order is the catalog order, so the result does not depend on hash order or on the order in which the caller listed the nodes. Identical subtrees are merged into one node, which is how an idempotence rule such as `p and p` can recognize its operands. The engine repeats passes until one changes nothing. That state is a fixed point.

The outcome is one of eight statuses in `RewriteStatus`:

| Status | Meaning |
| --- | --- |
| `Unchanged` | The input was already a fixed point. The report carries the input as its output. |
| `Normalized` | A changed fixed point was reached. An input whose identical subtrees were merged without a rule application also reports this status. |
| `BudgetExhausted` | A configured ceiling stopped the attempt. `exhausted_budget` names it. |
| `NonConvergent` | A complete formula state reappeared before a fixed point. |
| `Failed` | An internal consistency or evaluator failure prevented completion. |
| `InvalidInput` | The document failed structural validation. |
| `UnsupportedProfile` | No enabled rule set exists for the input profile. |
| `UnresolvedBinding` | A contextual request named a proposition that the signal catalog does not bind. |

Only `Unchanged` and `Normalized` carry an output document. Every other status has no output document. It carries a detail string and, when a complete state was reached, a digest of that state.

## Budgets

`RewriteOptions` holds a strategy (`BottomUpFirstMatch`) and four `RewriteBudgets` ceilings. All four are deterministic counts, never wall-clock time.

| Budget | Default | Counts |
| --- | ---: | --- |
| `max_iterations` | 32 | Complete rewrite passes. |
| `max_nodes` | 100,000 | Nodes in any rebuilt candidate graph. |
| `max_rule_applications` | 100,000 | Output-changing rule applications. |
| `max_work_units` | 1,000,000 | Node visits and emissions. |

The options are part of the request digest, so a replay uses the same ceilings.

## Reports

A `RewriteReport` records everything needed to repeat and check the attempt: the caller's formula identity, the engine source revision the caller supplied, the catalog digest, the input digest, a request digest over all of them, the options, the counters, and the ordered `steps`. Each `RewriteStep` names the rule and its revision, the node it fired on, the source span of that node when the input had one, a digest of the node before and after, and a rolling digest over every earlier step.

Rewrites keep the input's schema edition, semantic profile, caller formula identity and source revision. Source spans on nodes that survive are kept.

`rewrite_with_context` adds a `SignalCatalogDocument` and an optional `RequirementContextDocument`. It checks before and after rewriting that every proposition in the formula has a binding. The report then uses the `tl-rewrite.report/v2` schema and records the catalog digest and the context. A missing binding gives `UnresolvedBinding` with a `BindingFailure` that names the proposition and whether the input or the output failed the check.

## Profiles

| Profile | Entry point | Catalog |
| --- | --- | --- |
| `mltl.closed-trace/v1` | `rewrite` | `catalog()` |
| `mltl.origin-complete-history/v1` | `rewrite` | `past_catalog()` |
| `mltl.infinite-trace/v1` | `rewrite_infinite` | `infinite_catalog()` |

`rewrite` picks the catalog from the document's profile. The online-prefix profile is refused: its results can stay pending, and no population of prefix-equivalence evidence has been retained for it. `rewrite_infinite` takes a tl-syntax unbounded formula document, an optional fairness-premises document and a report byte ceiling. It remaps the ordered fairness roots to the rewritten graph and exposes no output unless it reaches a fixed point. The rule families are in [rules.md](rules.md).

## Semantics of the supported finite profile

The closed-trace profile pads missing proposition observations with false, and Boolean constants hold at every queried instant, including after the last observation. As a result `F[a,b] true` is true and `G[a,b] false` is false even on traces shorter than `a + 1`. This differs from textbook finite-trace conventions that require the lower-bound instant to exist. Equivalence claims cover the declared profile and the horizon-complete domain only. [replay-and-evidence.md](replay-and-evidence.md) defines that domain.

## Where the crate sits

A formula passes through several crates on its way from text to a Quire outcome. tl-rewrite is the step between parsing and evaluation.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="images/pipeline-dark.svg">
  <img alt="A formula moves from text to a Quire outcome. tl-parse turns MLTL text into a formula document, tl-rewrite returns a rewritten document, and tl-mltl evaluates it against a trace. In tests, tl-oracle evaluates the same formula and trace independently and the verdicts are compared. quire-mltl maps the result to a Boolean Value or a typed NonValue reason. All documents use the vocabulary of tl-syntax." src="images/pipeline-light.svg">
</picture>

The calling program moves documents from one stage to the next. The dependency structure between the crates is in the [README](../README.md#the-tl-stack).
