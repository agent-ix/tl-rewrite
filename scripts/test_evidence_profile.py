#!/usr/bin/env python3
"""Behavior tests for qualification profiles and bound retractions."""

from __future__ import annotations

import importlib.util
import json
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location(
    "evidence_profile", ROOT / "scripts" / "evidence_profile.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def main() -> int:
    active = ROOT / "evidence" / "tl-rewrite-v01-1b08a6c9e7bc-20260831T203039Z"
    legacy = ROOT / "evidence" / "tl-rewrite-v01-4b716afa9d0f-20260831T145617Z"
    assert MODULE.resolve_profile(active) == "inconclusive"
    assert MODULE.resolve_profile(legacy) == "retracted"
    assert legacy.name in MODULE.retracted_records()
    assert active.name not in MODULE.retracted_records(), "assured record was retracted"

    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        evidence = root / "evidence"
        evidence.mkdir()
        shutil.copytree(legacy, evidence / legacy.name)
        shutil.copy2(legacy.with_suffix(".sha256"), evidence / legacy.with_suffix(".sha256").name)
        registry = json.loads((ROOT / "evidence" / "RETRACTIONS.json").read_text())
        registry["records"] = {legacy.name: registry["records"][legacy.name]}
        registry_path = evidence / "RETRACTIONS.json"
        registry_path.write_text(json.dumps(registry), encoding="utf-8")
        assert MODULE.retracted_records(root) == {legacy.name}
        registry["records"][legacy.name]["manifestSha256"] = "0" * 64
        registry_path.write_text(json.dumps(registry), encoding="utf-8")
        try:
            MODULE.retracted_records(root)
        except ValueError:
            pass
        else:
            raise AssertionError("retraction manifest digest mutation was accepted")

    with tempfile.TemporaryDirectory(prefix="tl-rewrite-profile-") as directory:
        record = Path(directory)
        (record / "source-revision.txt").write_text(
            "92d3c45a2f01ff744b353d434c53b99823a71234\n", encoding="utf-8"
        )
        (record / "collection-input.json").write_text("{}\n", encoding="utf-8")
        assert MODULE.resolve_profile(record) == "inconclusive"
        (record / "source-revision.txt").write_text(
            "1b08a6c9e7bca9ebd2ff3831dfc5db9a5749dee4\n", encoding="utf-8"
        )
        try:
            MODULE.resolve_profile(record)
        except ValueError:
            pass
        else:
            raise AssertionError("v2-era profile omission was accepted")
        (record / "collection-input.json").write_text(
            json.dumps({"qualificationProfile": "fabricated/v9"}), encoding="utf-8"
        )
        try:
            MODULE.resolve_profile(record)
        except ValueError:
            pass
        else:
            raise AssertionError("unknown qualification profile was accepted")
        current_revision = subprocess.run(
            ["/usr/bin/git", "rev-parse", "HEAD"], cwd=ROOT, check=True,
            capture_output=True, text=True,
        ).stdout.strip()
        expected = MODULE.tool_identity.validate_lock(
            json.loads((ROOT / "tools.lock").read_text(encoding="utf-8"))
        )
        (record / "source-revision.txt").write_text(current_revision + "\n", encoding="utf-8")
        (record / "collection-input.json").write_text(
            json.dumps({
                "qualificationProfile": MODULE.QUALIFICATION_V2,
                "tools": {
                    "identities": expected["tools"],
                    "runtimeIdentities": expected["runtimeIdentities"],
                },
            }),
            encoding="utf-8",
        )
        assert MODULE.resolve_profile(record) == "v2"
        (record / "collection-input.json").write_text(
            json.dumps({"qualificationProfile": MODULE.QUALIFICATION_V2}), encoding="utf-8"
        )
        assert MODULE.resolve_profile(record) == "inconclusive"
    print("evidence qualification profile behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
