#!/usr/bin/env python3
"""Verify the exact executable identities used for local qualification."""

from __future__ import annotations

import hashlib
import json
import shutil
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
LOCK = ROOT / "tools.lock"
REQUIRED = ("bash", "cargo", "git", "make", "node", "python3", "quire", "rustc", "sha256sum")
RUNTIME_REQUIRED = ("cargo", "rustc")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validated_identities(value: Any, names: tuple[str, ...], label: str) -> dict[str, dict[str, str]]:
    if not isinstance(value, dict) or set(value) != set(names):
        raise ValueError(f"tools.lock does not contain the exact {label} census")
    validated: dict[str, dict[str, str]] = {}
    for name in names:
        identity = value.get(name)
        if not isinstance(identity, dict) or set(identity) != {"path", "sha256"}:
            raise ValueError(f"tools.lock has a malformed {label} identity for {name}")
        path = identity.get("path")
        digest = identity.get("sha256")
        if not isinstance(path, str) or not Path(path).is_absolute():
            raise ValueError(f"tools.lock path for {name} is not absolute")
        if not isinstance(digest, str) or len(digest) != 64 or any(
            character not in "0123456789abcdef" for character in digest
        ):
            raise ValueError(f"tools.lock digest for {name} is malformed")
        validated[name] = {"path": path, "sha256": digest}
    return validated


def validate_lock(value: Any) -> dict[str, dict[str, dict[str, str]]]:
    if not isinstance(value, dict) or value.get("schemaVersion") not in {
        "tl-rewrite.qualified-tools/v1", "tl-rewrite.qualified-tools/v2"
    }:
        raise ValueError("tools.lock has an unknown schema")
    if value["schemaVersion"] == "tl-rewrite.qualified-tools/v1":
        legacy_required = tuple(name for name in REQUIRED if name != "node")
        return {"tools": validated_identities(value.get("tools"), legacy_required, "legacy tool")}
    environment = value.get("environment")
    if not isinstance(environment, dict) or set(environment) != {
        "cargoTargetDir", "home", "rustupToolchain"
    }:
        raise ValueError("tools.lock has a malformed qualification environment")
    for name in ("home", "cargoTargetDir"):
        if not isinstance(environment.get(name), str) or not Path(environment[name]).is_absolute():
            raise ValueError(f"tools.lock qualification {name} is not absolute")
    if not isinstance(environment.get("rustupToolchain"), str) or not environment["rustupToolchain"]:
        raise ValueError("tools.lock qualification rustupToolchain is empty")
    return {
        "tools": validated_identities(value.get("tools"), REQUIRED, "mandatory-tool"),
        "runtimeIdentities": validated_identities(
            value.get("runtimeIdentities"), RUNTIME_REQUIRED, "Rust runtime"
        ),
    }


def load_lock(path: Path = LOCK) -> tuple[dict[str, Any], dict[str, dict[str, dict[str, str]]]]:
    value = json.loads(path.read_text(encoding="utf-8"))
    return value, validate_lock(value)


def trusted_path(tools: dict[str, dict[str, str]]) -> str:
    parents: list[str] = []
    for name in REQUIRED:
        parent = str(Path(tools[name]["path"]).parent)
        if parent not in parents:
            parents.append(parent)
    return ":".join(parents)


def verify_live(
    value: dict[str, Any], identities: dict[str, dict[str, dict[str, str]]]
) -> tuple[list[str], list[str]]:
    unavailable: list[str] = []
    mismatches: list[str] = []
    tools = identities["tools"]
    search_path = trusted_path(tools)
    for name in REQUIRED:
        expected = tools[name]
        locked_path = Path(expected["path"])
        try:
            locked_digest = sha256(locked_path)
        except OSError as error:
            unavailable.append(f"cannot read locked tool {name}: {error}")
            continue
        if locked_digest != expected["sha256"]:
            mismatches.append(
                f"locked tool digest mismatch for {name}: expected {expected['sha256']}, "
                f"got {locked_digest}"
            )
            continue
        observed = shutil.which(name, path=search_path)
        if observed is None:
            unavailable.append(f"qualified tool is unavailable: {name}")
            continue
        if observed != expected["path"]:
            mismatches.append(
                f"qualified tool path mismatch for {name}: expected {expected['path']}, got {observed}"
            )
            continue
        try:
            observed_digest = sha256(Path(observed))
        except OSError as error:
            unavailable.append(f"cannot read qualified tool {name}: {error}")
            continue
        if observed_digest != expected["sha256"]:
            mismatches.append(
                f"qualified tool digest mismatch for {name}: expected {expected['sha256']}, "
                f"got {observed_digest} at {observed}"
            )
    for name, expected in identities.get("runtimeIdentities", {}).items():
        try:
            observed_digest = sha256(Path(expected["path"]))
        except OSError as error:
            unavailable.append(f"cannot read qualified Rust runtime {name}: {error}")
            continue
        if observed_digest != expected["sha256"]:
            mismatches.append(
                f"qualified Rust runtime digest mismatch for {name}: "
                f"expected {expected['sha256']}, got {observed_digest}"
            )
    return unavailable, mismatches


def main() -> int:
    if len(sys.argv) != 2 or sys.argv[1] not in {"--verify-live", "--trusted-path", "--home"}:
        print("usage: tool_identity.py {--verify-live|--trusted-path|--home}", file=sys.stderr)
        return 2
    try:
        value, identities = load_lock()
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"qualified tool lock is unavailable: {error}", file=sys.stderr)
        return 2
    if sys.argv[1] == "--trusted-path":
        print(trusted_path(identities["tools"]))
        return 0
    if sys.argv[1] == "--home":
        print(value["environment"]["home"])
        return 0
    unavailable, mismatches = verify_live(value, identities)
    for error in unavailable + mismatches:
        print(error, file=sys.stderr)
    if unavailable:
        return 2
    return 1 if mismatches else 0


if __name__ == "__main__":
    raise SystemExit(main())
