#!/usr/bin/env python3
"""Portable behavior tests for the host-scoped qualification lock."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location("tool_identity", ROOT / "scripts" / "tool_identity.py")
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        bin_dir = root / "bin"
        runtime_dir = root / "runtime"
        bin_dir.mkdir()
        runtime_dir.mkdir()
        tools = {}
        for name in MODULE.REQUIRED:
            path = bin_dir / name
            path.write_text(f"#!/bin/sh\necho {name}\n", encoding="utf-8")
            path.chmod(0o755)
            tools[name] = {"path": str(path), "sha256": digest(path)}
        runtimes = {}
        for name in MODULE.RUNTIME_REQUIRED:
            path = runtime_dir / name
            path.write_text(f"qualified {name}\n", encoding="utf-8")
            runtimes[name] = {"path": str(path), "sha256": digest(path)}
        value = {
            "schemaVersion": "tl-rewrite.qualified-tools/v2",
            "environment": {
                "cargoTargetDir": str(root / "target"),
                "home": str(root / "home"),
                "rustupToolchain": "test-toolchain",
            },
            "runtimeIdentities": runtimes,
            "tools": tools,
        }
        lock = root / "tools.lock"
        lock.write_text(json.dumps(value), encoding="utf-8")
        loaded, identities = MODULE.load_lock(lock)
        assert loaded == value
        assert MODULE.verify_live(loaded, identities) == ([], [])

        shadow = root / "shadow"
        shadow.mkdir()
        fake = shadow / "cargo"
        fake.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
        fake.chmod(0o755)
        old_path = os.environ.get("PATH")
        try:
            os.environ["PATH"] = f"{shadow}:{old_path or ''}"
            assert MODULE.verify_live(loaded, identities) == ([], [])
        finally:
            if old_path is None:
                os.environ.pop("PATH", None)
            else:
                os.environ["PATH"] = old_path

        identities["tools"]["cargo"]["sha256"] = "0" * 64
        assert MODULE.verify_live(loaded, identities)[1]
        try:
            MODULE.validate_lock({**value, "environment": {"home": "relative"}})
        except ValueError:
            pass
        else:
            raise AssertionError("malformed qualification environment was accepted")
    print("qualified tool identity behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
