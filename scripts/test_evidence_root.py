#!/usr/bin/env python3
"""Behavior tests for the evidence-root census and live marker."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location("check_evidence_root", ROOT / "scripts" / "check_evidence_root.py")
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def main() -> int:
    with tempfile.TemporaryDirectory() as directory:
        evidence = Path(directory)
        (evidence / "ANCHORS").write_text("", encoding="utf-8")
        orphan = evidence / "tl-rewrite-v01-deadbeefcafe-20000101T000000Z"
        orphan.mkdir()
        assert MODULE.check(evidence)
        (orphan / ".collecting").write_text("", encoding="utf-8")
        assert MODULE.check(evidence), "empty marker bypassed the census"
        unknown = evidence / "fabricated.txt"
        unknown.write_text("x", encoding="utf-8")
        assert any("unrecognized" in error for error in MODULE.check(evidence))
        unknown.unlink()
        script = subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "check_evidence_root.py"), str(evidence)],
            check=False, capture_output=True,
        )
        assert script.returncode != 0, "evidence-root gate exit contract accepted an orphan"
    print("evidence-root census behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
