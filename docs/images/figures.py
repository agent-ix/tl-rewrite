# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (C) 2026 Agent-IX
"""tl-rewrite documentation figures, rendered by the docs-figures skill.

Regenerate from the repository root with the agent-ix/dev-tools skill:

    python3 -m pip install fonttools
    python3 <dev-tools>/skills/docs-figures/scripts/render.py docs/images/figures.py

Add --check to confirm the committed SVGs are current. Counts are read from
this repository: the rule corpus (corpus/rules/manifest.json) and the
counterexample corpus (corpus/counterexamples/manifest.json).

The block between the two SHARED markers is identical in every TL repository
(tl-syntax, tl-parse, tl-rewrite, tl-mltl, tl-oracle, quire-mltl). Copy it
verbatim and call tl_stack(here="<crate>") and stack_pipeline(here="<crate>").
"""

import json
from pathlib import Path

from docs_figures import FONTS, SMALL, TITLE, Figure  # noqa: F401

STYLE = "ix-docs"

ROOT = Path(__file__).resolve().parents[2]

# One role per kind of thing, fixed across the TL and Quire repositories.
ROLES = [
    ("code", "authored source", "box"),
    ("logic", "processing step", "box"),
    ("model", "checked document / view", "box"),
    ("evidence", "outcome", "box"),
    ("ink", "input", "box"),
]

# ==== SHARED: copy verbatim into sibling figures.py (begin) ===================

TL_CRATES = [
    # (column, row, name, one-line role, dependency note)
    (0, 1, "tl-parse", "text to formula graph"),
    (1, 1, "tl-oracle", "independent reference"),
    (2, 1, "tl-mltl", "evaluates traces"),
    (3, 1, "quire-mltl", "the bridge to Quire"),
    (2, 2, "tl-rewrite", "equivalent rewrites"),
    (3, 2, "quire-observation", "Quire observations"),
]


def tl_stack(here="tl-syntax"):
    """The TL crate family as a dependency graph; `here` is drawn highlighted."""
    f = Figure(
        "tl-stack",
        980,
        356,
        "The TL crates. tl-syntax defines the formula graph, profiles and signal catalog, and "
        "tl-parse, tl-oracle, tl-mltl, tl-rewrite and quire-mltl all depend on it. tl-rewrite "
        "depends on tl-mltl. quire-mltl depends on tl-mltl and quire-observation, and is the "
        "single bridge to Quire. Dashed lines are development-only: tl-mltl and tl-rewrite "
        "test against tl-oracle, and tl-rewrite tests against tl-parse.",
    )
    w, h, step = 192, 56, 262
    rows = {0: 24, 1: 140, 2: 256}
    f.text(3 * step, 244, "QUIRE", "muted", "mono", SMALL)
    f.box(0, rows[0], 980, h, "logic" if here == "tl-syntax" else "ink", "tl-syntax",
          "formula graph · semantic profiles · signal catalog · strict documents")
    where = {}
    for col, row, name, sub in TL_CRATES:
        x, y = col * step, rows[row]
        where[name] = (x, y)
        role = "logic" if name == here else "ink"
        f.box(x, y, w, h, role, name, sub, dashed=(name == "quire-observation"))
    for name in ("tl-parse", "tl-oracle", "tl-mltl", "quire-mltl"):
        x, y = where[name]
        f.line([(x + w / 2, rows[0] + h), (x + w / 2, y - 3)], "model")
    f.note(where["tl-parse"][0] + w / 2 + 10, 104, "types", "model")
    mx, my = where["tl-mltl"]
    rx, ry = where["tl-rewrite"]
    f.line([(mx + w / 2, my + h), (rx + w / 2, ry - 3)], "model")
    f.note(mx + w / 2 + 10, my + h + 30, "documents", "model")
    qx, qy = where["quire-mltl"]
    f.line([(mx + w, my + h / 2), (qx - 3, qy + h / 2)], "model")
    f.note(mx + w + 8, my + h / 2 - 8, "results", "model")
    ox, oy = where["quire-observation"]
    f.line([(ox + w / 2, oy - 3), (ox + w / 2, qy + h)], "model")
    f.note(ox + w / 2 + 10, qy + h + 30, "observations", "model")
    # development-only edges: tl-oracle and tl-parse feed tests of tl-mltl and tl-rewrite
    px, py = where["tl-parse"]
    zx, zy = where["tl-oracle"]
    f.line([(zx + w, zy + h / 2), (mx - 3, my + h / 2)], "model", dashed=True)
    f.line([(zx + w / 2, zy + h), (zx + w / 2, ry + 18), (rx - 3, ry + 18)], "model", dashed=True)
    f.line([(px + w / 2, py + h), (px + w / 2, ry + 40), (rx - 3, ry + 40)], "model", dashed=True)
    f.legend(0, 350, [("logic", "this repository", "box"), ("ink", "other crate", "box"),
                      ("model", "depends on", "solid"), ("model", "development only", "dash")])
    return f


PIPELINE_STAGES = [
    ("MLTL text", "your formula", "code", None),
    ("tl-parse", "parse", "logic", "tl-parse"),
    ("tl-rewrite", "rewrite", "logic", "tl-rewrite"),
    ("tl-mltl", "evaluate", "logic", "tl-mltl"),
    ("tl-oracle", "tests only", "logic", "tl-oracle"),
    ("quire-mltl", "map to Quire", "logic", "quire-mltl"),
]


def stack_pipeline(here="tl-syntax"):
    """End-to-end flow: text, parse, rewrite, evaluate with an oracle cross-check, Quire view."""
    f = Figure(
        "pipeline",
        980,
        392,
        "A formula moves from text to a Quire outcome. tl-parse turns MLTL text into a formula "
        "document, tl-rewrite returns a rewritten document, and tl-mltl evaluates it against a "
        "trace. In tests, tl-oracle evaluates the same formula and trace independently and the "
        "verdicts are compared. quire-mltl maps the result to a Boolean Value or a typed "
        "NonValue reason. All documents use the vocabulary of tl-syntax.",
    )
    w, h = 140, 56
    step = (980 - w) / (len(PIPELINE_STAGES) - 1)
    cx = [round(i * step + w / 2) for i in range(len(PIPELINE_STAGES))]
    for i, (name, sub, role, crate) in enumerate(PIPELINE_STAGES):
        if crate is not None:
            role = "logic" if crate == here else "ink"
        f.box(cx[i] - w // 2, 0, w, h, role, name, sub)
        f.line([(cx[i], h + 2), (cx[i], 312)], "faint", dashed=True, arrow=False, weight=1.0)
    msgs = [
        (0, 1, 96, "MLTL text", "code", False),
        (1, 2, 140, "formula document", "model", False),
        (2, 3, 184, "rewritten formula", "model", False),
        (3, 4, 228, "formula + trace", "model", True),
        (4, 3, 262, "reference verdict", "evidence", True),
        (3, 5, 306, "result", "model", False),
    ]
    for a, b, y, label, role, dev in msgs:
        sign = 1 if b > a else -1
        f.line([(cx[a] + 3 * sign, y), (cx[b] - 3 * sign, y)], role, dashed=dev,
               weight=2.4 if not dev else 1.5)
        left = min(cx[a], cx[b]) + 10
        f.note(left, y - 8, label, role)
    f.box(980 - 190, 332, 190, 50, "evidence", "Value or NonValue", "true / false or reason")
    f.line([(cx[5], 312), (cx[5], 329)], "evidence")
    f.box(0, 332, 520, 50, "logic" if here == "tl-syntax" else "ink", "tl-syntax",
          "the formula, profile, signal and document types every step uses")
    return f


# ==== SHARED (end) ============================================================


# ---- repository data ---------------------------------------------------------


def load(path):
    return json.loads((ROOT / path).read_text(encoding="utf-8"))


def rule_counts():
    """Enabled and excluded rules of the future catalog, and enabled rules per family."""
    cases = load("corpus/rules/manifest.json")["cases"]
    families = {}
    for case in cases:
        if case["disposition"] == "enabled":
            family = case["id"].split(".")[0]
            families[family] = families.get(family, 0) + 1
    excluded = [c["id"] for c in cases if c["disposition"] == "excluded"]
    enabled = sum(families.values())
    return enabled, len(excluded), families


def counterexample_counts():
    counts = load("corpus/counterexamples/manifest.json")["counts"]
    return counts["mismatch"], counts["total"]


# ---- figures -----------------------------------------------------------------


def hero():
    """README hero: a rule catalog rewrites a formula to a normal form; replay and equivalence check it."""
    enabled, excluded, _ = rule_counts()
    mismatch, total = counterexample_counts()
    f = Figure(
        "hero",
        980,
        540,
        f"A formula document goes into rewrite, which applies the {enabled} enabled rules of the "
        f"catalog ({excluded} more are excluded) bottom-up, first match, under explicit budgets, and "
        f"returns a RewriteReport whose output is the normal form. replay re-runs the request and "
        f"reports Verified or Mismatch. check_equivalence compares the original and the normal form "
        f"on every trace within the horizon and reports Equivalent, or Mismatch with the first "
        f"counterexample trace.",
    )
    f.box(270, 0, 240, 56, "model", "rule catalog", f"{enabled} enabled · {excluded} excluded")
    f.line([(390, 59), (390, 109)], "model")
    f.note(400, 88, "rules in priority order", "model")
    f.box(0, 112, 200, 62, "ink", "formula document", "tl-syntax v1 or v2", "closed-trace profile")
    f.box(270, 112, 240, 62, "logic", "rewrite", "bottom-up · first match", "iteration · node · work budgets")
    f.box(580, 112, 200, 62, "model", "RewriteReport", "status · steps · digests")
    f.box(850, 112, 130, 62, "evidence", "normal form", "fixed point")
    f.line([(200, 143), (267, 143)], "ink", weight=2.4)
    f.note(206, 135, "input", "ink")
    f.line([(510, 143), (577, 143)], "logic", weight=2.4)
    f.note(516, 135, "steps", "logic")
    f.line([(780, 143), (847, 143)], "model", weight=2.4)
    f.note(786, 135, "output", "model")

    f.text(0, 226, "REPLAY", "muted", "mono", SMALL)
    f.box(580, 232, 200, 62, "logic", "replay", "re-runs the request", "compares every field")
    f.line([(680, 177), (680, 229)], "model")
    f.note(690, 208, "report", "model")
    f.box(270, 232, 240, 62, "evidence", "ReplayStatus", "Verified or Mismatch")
    f.line([(577, 263), (513, 263)], "logic")
    f.note(520, 255, "status", "logic")

    f.text(0, 340, "EQUIVALENCE", "muted", "mono", SMALL)
    f.box(270, 350, 240, 62, "logic", "check_equivalence", "every trace in the horizon", "evaluated by tl-mltl")
    f.line([(100, 177), (100, 381), (267, 381)], "ink")
    f.note(108, 373, "original", "ink")
    f.line([(915, 177), (915, 365), (513, 365)], "evidence")
    f.note(700, 357, "rewritten", "evidence")
    f.box(270, 452, 340, 56, "evidence", "ConformanceStatus",
          "Equivalent, or Mismatch with a trace")
    f.line([(390, 415), (390, 449)], "logic")
    f.legend(0, 534, [ROLES[0], ROLES[1], ROLES[2], ROLES[3], ROLES[4]])
    return f


def tl_stack_here():
    return tl_stack(here="tl-rewrite")


def pipeline_here():
    return stack_pipeline(here="tl-rewrite")


def rules():
    """Enabled rules of the future catalog by family, counted from the rule corpus."""
    enabled, excluded, families = rule_counts()
    order = [("bool", "Boolean"), ("neg", "negation"), ("temporal", "temporal")]
    values = [families[key] for key, _ in order]
    labels = [name for _, name in order]
    f = Figure(
        "rules",
        980,
        236,
        f"The future catalog enables {enabled} rules: "
        + ", ".join(f"{families[k]} {n}" for k, n in order)
        + f". {excluded} more are retained for review and excluded from execution.",
    )
    f.text(0, 16, "ENABLED RULES PER FAMILY", "muted", "mono", SMALL)
    f.columns(0, 30, 980, 202, values, labels=labels, role="logic")
    return f


FIGURES = [hero, tl_stack_here, pipeline_here, rules]
