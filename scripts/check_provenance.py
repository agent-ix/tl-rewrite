#!/usr/bin/env python3
"""Check that this repository's stated provenance is the provenance it compiles (FR-006-AC-2).

This is a producer. It reads bytes that are already in the tree and reports one
structured row per obligation. It runs no compiler, no test and no solver, it
computes no aggregate verdict beyond its own exit status, and it retains nothing.

Three obligations, each of which has been wrong in this repository before.

**The corpora are the bytes they claim to be.** `corpus/west-v1/SHA256SUMS` pins
every retained WEST byte; `corpus/rules` and `corpus/counterexamples` are the
oracles the rewrite producers are checked against, and an oracle nobody digests
is an oracle anyone can edit.

**The declared dependency revisions are the compiled ones.** `TL_SYNTAX_REVISION`
and `TL_MLTL_REVISION` are wire fields: every `ConformanceReport` this crate
emits names them as the revisions the comparison ran against. Before this change
`TL_MLTL_REVISION` said `da2c7704`, a revision reachable from no branch or tag,
while `Cargo.toml` pinned `fe1c620d` — so every conformance report in the
repository attributed its verdicts to an evaluator that did not produce them.
Nothing noticed, because the only test compared the constant to itself. The
constants, `Cargo.toml` and `Cargo.lock` are now required to agree.

**The corpus revision is the declared upstream revision.** `WEST_REVISION` and
`corpus/west-v1/manifest.json` name the same upstream commit.

Exit status: 0 when every row passed, 1 when a row failed, 2 on a usage or
environment error — which is a different fact from a failing check.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
WEST = ROOT / "corpus" / "west-v1"
CHECKSUM_LINE = re.compile(r"^([0-9a-f]{64})  ([^/]+)$")

# The constants this crate publishes as wire fields, and where each one's truth
# lives. `Cargo.lock` is included because a lockfile that drifted from the
# manifest is how a build silently uses a revision nobody declared.
DEPENDENCIES = {
    "TL_SYNTAX_REVISION": "tl-syntax",
    "TL_MLTL_REVISION": "tl-mltl",
}

# Corpora whose bytes are an oracle for a producer. Each is digested here so that
# an edited oracle is a reported difference rather than a silently moved target.
ORACLES = (
    "corpus/rules/manifest.json",
    "corpus/counterexamples/manifest.json",
    "corpus/west-v1/manifest.json",
)


class ProvenanceError(RuntimeError):
    """The check could not be performed. Distinct from a check that failed."""


def sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def read(relative: str) -> str:
    path = ROOT / relative
    if not path.is_file():
        raise ProvenanceError(f"{relative} is absent; there is nothing to check")
    return path.read_text(encoding="utf-8")


def row(symbol: str, outcome: str, detail: str, **extra: Any) -> dict[str, Any]:
    return {
        "protocol": "tl-rewrite.provenance-integrity/v1",
        "symbol": symbol,
        "outcome": outcome,
        "domainOutcome": outcome if outcome != "pass" else "consistent",
        "traceIds": ["TC-016", "NFR-002-AC-2", "FR-004-AC-3"],
        "detail": detail,
        **extra,
    }


def corpus_rows() -> list[dict[str, Any]]:
    """Re-hash every retained WEST byte against the pinned checksum manifest."""
    manifest = WEST / "SHA256SUMS"
    if not manifest.is_file():
        raise ProvenanceError("corpus/west-v1/SHA256SUMS is absent")
    lines = [line for line in manifest.read_text(encoding="utf-8").splitlines() if line.strip()]
    if not lines:
        # A checksum manifest with no lines verifies nothing while returning
        # success. That is vacuous, and vacuous is not passed.
        return [
            row(
                "corpus/west-v1/SHA256SUMS",
                "vacuous",
                "the WEST checksum manifest lists no artifacts, so it pins nothing",
            )
        ]
    listed = set()
    rows = []
    for number, line in enumerate(lines, start=1):
        match = CHECKSUM_LINE.fullmatch(line)
        if match is None:
            rows.append(
                row(
                    f"corpus/west-v1/SHA256SUMS:{number}",
                    "malformed",
                    "the checksum line is not <sha256><two spaces><name>",
                )
            )
            continue
        expected, name = match.group(1), match.group(2)
        listed.add(name)
        path = WEST / name
        if path.is_symlink() or not path.is_file():
            rows.append(
                row(
                    f"corpus/west-v1/{name}",
                    "unavailable",
                    "the listed corpus artifact is absent or is a symlink",
                )
            )
            continue
        observed = sha256(path.read_bytes())
        if observed != expected:
            rows.append(
                row(
                    f"corpus/west-v1/{name}",
                    "fail",
                    f"retained bytes digest {observed}, the manifest pins {expected}",
                )
            )
            continue
        rows.append(
            row(
                f"corpus/west-v1/{name}",
                "pass",
                "retained bytes match the pinned digest",
                sha256=observed,
            )
        )
    # Membership in both directions. A manifest that lists a subset verifies only
    # what it lists, and an added file would otherwise never be seen.
    present = {
        item.name
        for item in WEST.iterdir()
        if item.is_file() and item.name != "SHA256SUMS"
    }
    for unlisted in sorted(present - listed):
        rows.append(
            row(
                f"corpus/west-v1/{unlisted}",
                "fail",
                "the corpus directory holds an artifact the checksum manifest does not list",
            )
        )
    return rows


def oracle_rows() -> list[dict[str, Any]]:
    """Digest every corpus that acts as an oracle for a producer."""
    rows = []
    for relative in ORACLES:
        path = ROOT / relative
        if not path.is_file():
            rows.append(row(relative, "unavailable", "the declared oracle is absent"))
            continue
        raw = path.read_bytes()
        try:
            parsed = json.loads(raw)
        except json.JSONDecodeError as error:
            rows.append(row(relative, "malformed", f"the oracle is not readable JSON: {error}"))
            continue
        cases = parsed.get("cases") or parsed.get("selectedCases")
        if not isinstance(cases, list) or not cases:
            rows.append(row(relative, "vacuous", "the oracle declares no cases"))
            continue
        rows.append(
            row(
                relative,
                "pass",
                f"the oracle declares {len(cases)} case(s)",
                sha256=sha256(raw),
                cases=len(cases),
            )
        )
    return rows


def dependency_rows() -> list[dict[str, Any]]:
    """Require the published revision constants to be the compiled revisions."""
    library = read("src/lib.rs")
    manifest = read("Cargo.toml")
    lockfile = read("Cargo.lock")
    rows = []
    for constant, crate in DEPENDENCIES.items():
        declared = re.search(rf'{constant}: &str = "([0-9a-f]{{40}})";', library)
        if declared is None:
            rows.append(
                row(
                    f"dependency:{crate}",
                    "fail",
                    f"src/lib.rs does not declare a 40-hex {constant}",
                )
            )
            continue
        revision = declared.group(1)
        pinned = re.search(
            rf'{re.escape(crate)} = \{{ git = "[^"]+", rev = "([0-9a-f]{{40}})"', manifest
        )
        if pinned is None:
            rows.append(
                row(f"dependency:{crate}", "fail", f"Cargo.toml does not pin {crate} by revision")
            )
            continue
        locked = re.search(
            rf'name = "{re.escape(crate)}"\nversion = "[^"]+"\nsource = "git\+[^"]*'
            rf'rev=([0-9a-f]{{40}})#',
            lockfile,
        )
        if locked is None:
            rows.append(
                row(f"dependency:{crate}", "fail", f"Cargo.lock does not resolve {crate} to a git revision")
            )
            continue
        if revision != pinned.group(1) or revision != locked.group(1):
            rows.append(
                row(
                    f"dependency:{crate}",
                    "fail",
                    (
                        f"{constant} is {revision}, Cargo.toml pins {pinned.group(1)}, and "
                        f"Cargo.lock resolves {locked.group(1)}; the wire field would attribute "
                        "a verdict to a revision that did not produce it"
                    ),
                )
            )
            continue
        rows.append(
            row(
                f"dependency:{crate}",
                "pass",
                f"{constant}, Cargo.toml and Cargo.lock all name {revision}",
                revision=revision,
            )
        )

    west = re.search(r'WEST_REVISION: &str = "([0-9a-f]{40})";', library)
    corpus = json.loads(read("corpus/west-v1/manifest.json"))
    if west is None:
        rows.append(row("corpus:west-revision", "fail", "src/lib.rs declares no WEST_REVISION"))
    elif west.group(1) != corpus.get("upstreamRevision"):
        rows.append(
            row(
                "corpus:west-revision",
                "fail",
                (
                    f"WEST_REVISION is {west.group(1)} but the corpus manifest names "
                    f"{corpus.get('upstreamRevision')}"
                ),
            )
        )
    else:
        rows.append(
            row(
                "corpus:west-revision",
                "pass",
                f"the retained corpus and the crate both name {west.group(1)}",
                revision=west.group(1),
            )
        )
    return rows


def build_report() -> dict[str, Any]:
    entries = corpus_rows() + oracle_rows() + dependency_rows()
    if not entries:
        raise ProvenanceError("no provenance obligation was evaluated at all")
    return {
        "schemaVersion": "tl-rewrite.provenance-integrity/v1",
        "entries": entries,
        "matched": all(entry["outcome"] == "pass" for entry in entries),
    }


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--json", action="store_true", help="emit the rows as JSON")
    arguments = parser.parse_args(argv[1:])
    try:
        report = build_report()
    except ProvenanceError as error:
        print(str(error), file=sys.stderr)
        return 2
    if arguments.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        for entry in report["entries"]:
            print(f"{entry['symbol']}: {entry['outcome']} ({entry['detail']})")
    if not report["matched"]:
        print("the declared provenance is not the compiled provenance", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
