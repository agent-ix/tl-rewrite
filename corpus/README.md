# Conformance corpora

`west-v1/formulas_d1.txt` and `west-v1/LICENSE` are byte copies from the
canonical MIT-licensed `zwang271/WEST` repository at commit
`21cd99ab2e6095a099dd179029cfdeb54268ad3f`. The source path and selected cases
are recorded in `manifest.json`; `SHA256SUMS` pins every retained byte.

The selected strings are manually represented as validated tl-syntax graphs in
the integration test because parsing is explicitly outside this crate. Each
rewrite is then exhaustively compared with the pinned tl-mltl evaluator over
its reported horizon-complete domain. This exercises permitted WEST validation
inputs without claiming WEST equivalence, universal proof, or monitor qualification.
