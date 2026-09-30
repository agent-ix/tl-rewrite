#!/usr/bin/env python3
"""Check that this repository's stated provenance is the provenance it compiles.

This is a producer. It reads bytes that are already in the tree and reports one
structured row per obligation. It runs no compiler, no test and no solver, it
computes no aggregate verdict beyond its own exit status, and it retains nothing.

Two obligations, each of which has been wrong in this repository before.

**The declared dependency revisions are the compiled ones.** `TL_SYNTAX_REVISION`
and `TL_MLTL_REVISION` are wire fields: every `ConformanceReport` this crate
emits names them as the revisions the comparison ran against. The
constants, `Cargo.toml` and `Cargo.lock` are now required to agree. Issue #35
once locked a second tl-syntax revision through a renamed dev-dependency (one
revision resolves since 0.3.0), so agreement means: the `[dependencies]` pin is
locked, every other locked revision is one a `[dev-dependencies]` entry
declares, and tl-mltl compiles the pinned tl-syntax.

**The corpus revision is the declared upstream revision.** `WEST_REVISION` and
`corpus/west-v1/manifest.json` name the same upstream commit.

Exit status: 0 when every row passed, 1 when a row failed, 2 on a usage or
environment error — which is a different fact from a failing check.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent

# The constants this crate publishes as wire fields, and where each one's truth
# lives. `Cargo.lock` is included because a lockfile that drifted from the
# manifest is how a build silently uses a revision nobody declared.
DEPENDENCIES = {
    "TL_SYNTAX_REVISION": "tl-syntax",
    "TL_MLTL_REVISION": "tl-mltl",
}

# Production dependencies that themselves compile a published crate. The
# revision they lock must be the published one, or a conformance verdict names
# a revision the evaluator did not run on.
PRODUCTION_CONSUMERS = {
    "tl-syntax": ("tl-mltl",),
}


class ProvenanceError(RuntimeError):
    """The check could not be performed. Distinct from a check that failed."""


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


def manifest_section(manifest: str, name: str) -> str:
    """Return the body of one top-level `[name]` table of Cargo.toml."""
    match = re.search(
        rf"^\[{re.escape(name)}\]\n(.*?)(?=^\[|\Z)", manifest, re.MULTILINE | re.DOTALL
    )
    return match.group(1) if match else ""


def inline_dependency(section: str, name: str) -> str | None:
    """Return one dependency's inline-table body independent of field order."""
    entry = re.search(
        rf"^{re.escape(name)}\s*=\s*\{{([^}}\n]*)\}}\s*(?:#.*)?$",
        section,
        re.MULTILINE,
    )
    return entry.group(1) if entry else None


def inline_string_field(table: str, name: str) -> str | None:
    """Return one quoted inline-table field, refusing absent or duplicate fields."""
    values = re.findall(rf'(?:^|,)\s*{re.escape(name)}\s*=\s*"([^"]*)"', table)
    return values[0] if len(values) == 1 else None


def development_revisions(manifest: str, crate: str) -> set[str]:
    """Revisions of `crate` that `[dev-dependencies]` declares, renamed or not."""
    revisions = set()
    for line in manifest_section(manifest, "dev-dependencies").splitlines():
        entry = re.match(r"^([A-Za-z0-9_-]+)\s*=", line)
        if entry is None:
            continue
        table = inline_dependency(line, entry.group(1))
        if table is None:
            continue
        package = inline_string_field(table, "package") or entry.group(1)
        if package != crate:
            continue
        revision = inline_string_field(table, "rev")
        if revision is not None and re.fullmatch(r"[0-9a-f]{40}", revision):
            revisions.add(revision)
    return revisions


def lock_packages(lockfile: str) -> list[dict[str, Any]]:
    """Parse Cargo.lock's `[[package]]` tables into name, git revision and dependencies.

    Python 3.10 has no `tomllib`, and Cargo writes these tables in one fixed
    shape, so each table is read field by field rather than matched as a span.
    """
    packages = []
    for table in lockfile.split("[[package]]\n")[1:]:
        name = re.search(r'^name = "([^"]+)"$', table, re.MULTILINE)
        if name is None:
            raise ProvenanceError("Cargo.lock holds a [[package]] table with no name")
        source = re.search(r'^source = "([^"]+)"$', table, re.MULTILINE)
        rev = re.search(r"[?&]rev=([0-9a-f]{40})#", source.group(1)) if source else None
        dependencies = re.search(r"^dependencies = \[\n(.*?)^\]$", table, re.MULTILINE | re.DOTALL)
        packages.append(
            {
                "name": name.group(1),
                "rev": rev.group(1) if rev else None,
                "dependencies": (
                    re.findall(r'^ "([^"]+)",$', dependencies.group(1), re.MULTILINE)
                    if dependencies
                    else []
                ),
            }
        )
    return packages


def consumer_revision(packages: list[dict[str, Any]], consumer: str, crate: str) -> str | None:
    """The locked revision of `crate` that the locked package `consumer` compiles against."""
    consumers = [package for package in packages if package["name"] == consumer]
    if len(consumers) != 1:
        return None
    locked = [package["rev"] for package in packages if package["name"] == crate]
    for dependency in consumers[0]["dependencies"]:
        if dependency == crate:
            # Cargo omits the qualifier only when one entry of the name is locked.
            return locked[0] if len(locked) == 1 else None
        if dependency.startswith(f"{crate} "):
            qualified = re.search(r"[?&]rev=([0-9a-f]{40})\)$", dependency)
            return qualified.group(1) if qualified else None
    return None


def dependency_rows() -> list[dict[str, Any]]:
    """Require the published revision constants to be the compiled revisions."""
    library = read("src/lib.rs")
    manifest = read("Cargo.toml")
    packages = lock_packages(read("Cargo.lock"))
    production = manifest_section(manifest, "dependencies")
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
        dependency = inline_dependency(production, crate)
        pinned = inline_string_field(dependency, "rev") if dependency is not None else None
        git = inline_string_field(dependency, "git") if dependency is not None else None
        if pinned is None or git is None or re.fullmatch(r"[0-9a-f]{40}", pinned) is None:
            rows.append(
                row(f"dependency:{crate}", "fail", f"Cargo.toml does not pin {crate} by revision")
            )
            continue
        locked_revisions = [
            package["rev"]
            for package in packages
            if package["name"] == crate and package["rev"] is not None
        ]
        if not locked_revisions:
            rows.append(
                row(f"dependency:{crate}", "fail", f"Cargo.lock does not resolve {crate} to a git revision")
            )
            continue
        if revision != pinned or pinned not in locked_revisions:
            rows.append(
                row(
                    f"dependency:{crate}",
                    "fail",
                    (
                        f"{constant} is {revision}, Cargo.toml pins {pinned}, and "
                        f"Cargo.lock resolves {', '.join(sorted(locked_revisions))}; the wire "
                        "field would attribute a verdict to a revision that did not produce it"
                    ),
                )
            )
            continue
        # A renamed dev-dependency can lock a second revision of the same package
        # (issue #35). Every other locked revision must be one a
        # `[dev-dependencies]` entry declares, so an undeclared second revision
        # is a failure rather than something the production pin hides.
        dev_revisions = development_revisions(manifest, crate)
        undeclared = sorted(
            locked for locked in set(locked_revisions) - {revision} if locked not in dev_revisions
        )
        if undeclared:
            rows.append(
                row(
                    f"dependency:{crate}",
                    "fail",
                    (
                        f"Cargo.lock also resolves {crate} at {', '.join(undeclared)}, which "
                        "neither the production pin nor any dev-dependency declares"
                    ),
                )
            )
            continue
        # The revision a production consumer compiles against is the one that
        # matters: if tl-mltl resolved the development tl-syntax, the evaluator
        # would run on a revision the wire field does not name.
        mismatched = []
        for consumer in PRODUCTION_CONSUMERS.get(crate, ()):
            resolved = consumer_revision(packages, consumer, crate)
            if resolved != revision:
                mismatched.append(f"{consumer} resolves {crate} at {resolved}")
        if mismatched:
            rows.append(
                row(
                    f"dependency:{crate}",
                    "fail",
                    (
                        f"{constant} is {revision} but {'; '.join(mismatched)}, so a production "
                        "consumer compiles a revision the wire field does not name"
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
    entries = dependency_rows()
    if not entries:
        raise ProvenanceError("no provenance obligation was evaluated at all")
    return {
        "schemaVersion": "tl-rewrite.provenance-integrity/v1",
        "entries": entries,
        "matched": all(entry["outcome"] == "pass" for entry in entries),
    }


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.parse_args(argv[1:])
    try:
        report = build_report()
    except ProvenanceError as error:
        print(str(error), file=sys.stderr)
        return 2
    for entry in report["entries"]:
        print(f"{entry['symbol']}: {entry['outcome']} ({entry['detail']})")
    if not report["matched"]:
        print("the declared provenance is not the compiled provenance", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
