#!/usr/bin/env python3
"""Reject unknown or orphaned entries at the retained-evidence root."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
ALLOWED_FILES = {"ANCHORS", "README.md", "RETRACTIONS.json"}


def check(evidence: Path) -> list[str]:
    errors: list[str] = []
    for entry in sorted(evidence.iterdir(), key=lambda path: path.name):
        if entry.is_symlink():
            errors.append(f"evidence root contains a symlink: {entry}")
        elif entry.is_file() and entry.name not in ALLOWED_FILES and entry.suffix != ".sha256":
            errors.append(f"unrecognized evidence-root file: {entry}")
        elif entry.is_dir() and not entry.with_suffix(".sha256").is_file():
            errors.append(f"retained evidence directory lacks a checksum manifest: {entry}")
    return errors


def main() -> int:
    evidence = Path(sys.argv[1]) if len(sys.argv) == 2 else ROOT / "evidence"
    if len(sys.argv) > 2:
        print("usage: check_evidence_root.py [EVIDENCE_ROOT]", file=sys.stderr)
        return 2
    try:
        errors = check(evidence)
    except OSError as error:
        print(f"cannot inspect evidence root: {error}", file=sys.stderr)
        return 2
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
