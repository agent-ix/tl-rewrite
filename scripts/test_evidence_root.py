#!/usr/bin/env python3
"""Behavior tests for the evidence-root census and live marker."""

from __future__ import annotations

import importlib.util
import os
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location("check_evidence_root", ROOT / "scripts" / "check_evidence_root.py")
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def main() -> int:
    revision = MODULE.current_revision()
    with tempfile.TemporaryDirectory() as directory:
        evidence = Path(directory)
        (evidence / "ANCHORS").write_text("", encoding="utf-8")
        orphan = evidence / "tl-rewrite-v01-deadbeefcafe-20000101T000000Z"
        orphan.mkdir()
        assert MODULE.check(evidence, revision, None)
        (orphan / ".collecting").write_text("", encoding="utf-8")
        assert MODULE.check(evidence, revision, None), "empty marker bypassed the census"
        unknown = evidence / "fabricated.txt"
        unknown.write_text("x", encoding="utf-8")
        assert any("unrecognized" in error for error in MODULE.check(evidence, revision, None))
        unknown.unlink()
        token = "test-token"
        (orphan / ".collecting").write_text(
            '{"pid": %d, "sourceRevision": "%s", "token": "%s"}\n'
            % (os.getpid(), revision, token), encoding="utf-8"
        )
        assert MODULE.check(evidence, revision, token) == []
    print("evidence-root census behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
