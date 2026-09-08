---
id: SR-016
title: "Failure-domain review of the complete tl-rewrite specification corpus"
type: SpecReview
analysis: failure-domain
scope: "spec/**/*.md"
review_set: all
---

# SR-016: Failure-domain review of the complete tl-rewrite specification corpus

## Summary

The corpus specifies identity, purity, bounded graph behavior, and explicit
non-success outcomes for the rewrite domain. It also documents one live
failure-domain hole: Make can suppress local gate failures without a local
control detecting that suppression.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1601 | high | NFR-003 records that `.IGNORE:` can make `make ci` report success after failed prerequisites and that no repository control detects Make execution controls; this leaves listed non-producer gates without the qualification-integrity failure boundary the NFR otherwise intends. | NFR-003, agent-ix/tl-rewrite#11 |
