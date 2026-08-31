#!/usr/bin/env python3
"""Bind the assurance argument to its retained record and executable anchor."""

from __future__ import annotations

import hashlib
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    assurance = (ROOT / "spec" / "assurance" / "AA-001.md").read_text(encoding="utf-8")
    record_match = re.search(r"`(evidence/tl-rewrite-v01-[^`]+)`", assurance)
    digest_match = re.search(r"checksum-manifest\s+SHA-256\s+`([0-9a-f]{64})`", assurance, re.DOTALL)
    if record_match is None or digest_match is None:
        print("assurance argument does not identify one retained record and digest", file=sys.stderr)
        return 1
    record = record_match.group(1)
    checksum = ROOT / f"{record}.sha256"
    if not checksum.is_file():
        print("assurance argument names a missing retained record", file=sys.stderr)
        return 1
    digest = sha256(checksum)
    if digest != digest_match.group(1):
        print("assurance argument does not bind its record's outer manifest digest", file=sys.stderr)
        return 1
    anchor = f"{digest}  {record}.sha256"
    anchors = (ROOT / "evidence" / "ANCHORS").read_text(encoding="utf-8").splitlines()
    if anchor not in anchors:
        print("assurance argument and evidence anchors disagree", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
