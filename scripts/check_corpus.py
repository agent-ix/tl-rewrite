#!/usr/bin/env python3
"""Verify the retained WEST corpus checksum manifest without a shell pipeline."""

from __future__ import annotations

import hashlib
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CORPUS = ROOT / "corpus" / "west-v1"
LINE = re.compile(r"^([0-9a-f]{64})  ([^/]+)$")


def main() -> int:
    corpus = Path(sys.argv[2]) if len(sys.argv) == 3 and sys.argv[1] == "--directory" else CORPUS
    if len(sys.argv) not in {1, 3}:
        return 2
    errors: list[str] = []
    for number, line in enumerate(
        (corpus / "SHA256SUMS").read_text(encoding="utf-8").splitlines(), start=1
    ):
        match = LINE.fullmatch(line)
        if match is None:
            errors.append(f"malformed corpus checksum line {number}")
            continue
        path = corpus / match.group(2)
        if not path.is_file() or path.is_symlink():
            errors.append(f"missing or unsafe corpus artifact: {path.name}")
            continue
        if hashlib.sha256(path.read_bytes()).hexdigest() != match.group(1):
            errors.append(f"corpus checksum mismatch: {path.name}")
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
