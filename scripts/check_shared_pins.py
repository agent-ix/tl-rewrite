#!/usr/bin/env python3
"""Observe local tools and classify them with native Engineering Assurance.

The executable embeds the reviewed matrix. This gate checks its structured
response and the installed EA version against the declared release. No removed
Python module or pre-release response digest is treated as consumed.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
PINS_PATH = ROOT / "assurance/pins.json"
REQUEST_PROTOCOL = "engineering-assurance.compatibility-request/v1"
RESULT_PROTOCOL = "engineering-assurance.compatibility-result/v1"
COMPONENTS = ("quire-cli", "quoin", "ix-flow", "engineering-assurance")
FORBIDDEN_REGISTRY = "npm.ix"
MIRROR_SCAN_FILES = (
    "requirements-assurance.txt", ".npmrc", "Cargo.toml", "Cargo.lock",
    "package.json", "package-lock.json", ".github/workflows/ci.yml", "Makefile",
)


class PinError(RuntimeError):
    """The installed classifier could not be used."""


def cli_path() -> str:
    return os.environ.get("ENGINEERING_ASSURANCE", "engineering-assurance")


def binary_attestation() -> dict[str, str]:
    executable = shutil.which(cli_path())
    if executable is None:
        raise PinError("Engineering Assurance executable is absent")
    path = Path(executable).resolve()
    try:
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        raise PinError(f"Engineering Assurance executable is unreadable: {error}") from error
    return {"path": str(path), "sha256": digest}


def observe(argv: list[str]) -> str | None:
    try:
        result = subprocess.run(argv, capture_output=True, text=True, timeout=60, check=False)
    except (OSError, ValueError, subprocess.TimeoutExpired):
        return None
    if result.returncode != 0:
        return None
    return result.stdout.strip() or None


def observe_quire() -> str | None:
    raw = observe(["quire", "provenance"])
    if raw is None:
        return None
    try:
        return json.loads(raw)["cli"]["version"]
    except (json.JSONDecodeError, KeyError, TypeError):
        return None


def observe_semver(argv: list[str]) -> str | None:
    raw = observe(argv)
    if raw is None:
        return None
    match = re.search(r"\b\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?\b", raw)
    return match.group(0) if match else None


def classify(observed: list[dict[str, str | None]]) -> tuple[dict[str, Any], bytes, int]:
    request = {"protocol": REQUEST_PROTOCOL, "observed": observed}
    try:
        completed = subprocess.run(
            [cli_path(), "compatibility"],
            input=json.dumps(request, separators=(",", ":")),
            capture_output=True, text=True, timeout=60, check=False,
        )
    except (OSError, ValueError, subprocess.TimeoutExpired) as error:
        raise PinError(f"Engineering Assurance classifier could not run: {error}") from error
    if completed.returncode not in (0, 1):
        raise PinError(
            f"Engineering Assurance classifier refused the request "
            f"(exit {completed.returncode}): {completed.stderr.strip()}"
        )
    try:
        result = json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise PinError(f"Engineering Assurance classifier returned invalid JSON: {error}") from error
    if not isinstance(result, dict):
        raise PinError("Engineering Assurance classifier result is not a JSON object")
    components = result.get("components")
    if (
        set(result) != {"protocol", "matrix_version", "outcome", "versions_compatible",
                        "human_acceptance_recorded", "gate_satisfied", "components"}
        or result.get("protocol") != RESULT_PROTOCOL
        or result.get("matrix_version") != "engineering-assurance.compatibility-matrix/v1"
        or type(result.get("versions_compatible")) is not bool
        or type(result.get("human_acceptance_recorded")) is not bool
        or type(result.get("gate_satisfied")) is not bool
        or not isinstance(components, list)
        or len(components) != len(COMPONENTS)
        or any(not isinstance(item, dict) for item in components)
        or {item.get("component") for item in components} != set(COMPONENTS)
        or any(
            set(item) != {"component", "observed", "expected", "verdict", "reason"}
            or not isinstance(item.get("expected"), str)
            or not item["expected"]
            or not isinstance(item.get("reason"), str)
            or not item["reason"]
            or
            item.get("verdict") not in ("compatible", "incompatible", "unknown")
            or item.get("observed") != next(
                observation["version"]
                for observation in observed
                if observation["component"] == item["component"]
            )
            for item in components
        )
        or result["versions_compatible"] != all(
            item["verdict"] == "compatible" for item in components
        )
        or result["gate_satisfied"] != (
            result["versions_compatible"] and result["human_acceptance_recorded"]
        )
        or result["outcome"] != (
            "compatible" if result["gate_satisfied"] else "withheld"
        )
        or (completed.returncode == 0) != result["gate_satisfied"]
    ):
        raise PinError("Engineering Assurance classifier returned an inconsistent result")
    return result, completed.stdout.encode("utf-8"), completed.returncode


def declared_version_mismatches(
    pins: dict[str, Any], observed: list[dict[str, str | None]]
) -> list[str]:
    """Bind the classifier observed on PATH to this repository's EA release pin."""
    declared = pins.get("engineering_assurance")
    if not isinstance(declared, dict):
        return ["engineering_assurance release declaration is absent"]
    version = declared.get("version")
    if not isinstance(version, str) or re.fullmatch(r"\d+\.\d+\.\d+", version) is None:
        return ["engineering_assurance version is absent or malformed"]
    mismatches = []
    installed = next((item["version"] for item in observed
                      if item["component"] == "engineering-assurance"), None)
    if installed != version:
        mismatches.append(f"installed Engineering Assurance {installed}, declared {version}")
    requirement = declared.get("requirement")
    if not isinstance(requirement, str) or f"--tag v{version}" not in requirement:
        mismatches.append("Engineering Assurance install command disagrees with declared version")
    module_install = declared.get("module_install")
    if not isinstance(module_install, str) or not module_install.endswith(f"@v{version}"):
        mismatches.append("Quoin module install command disagrees with declared version")
    return mismatches


def mirror_references(pins: dict[str, Any]) -> list[str]:
    offenders: list[str] = []
    for name in MIRROR_SCAN_FILES:
        path = ROOT / name
        if not path.is_file():
            continue
        try:
            source = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError) as error:
            offenders.append(f"{name}: unreadable ({error})")
            continue
        for number, line in enumerate(source.splitlines(), start=1):
            if FORBIDDEN_REGISTRY in line:
                offenders.append(f"{name}:{number}")
    if FORBIDDEN_REGISTRY in pins["engineering_assurance"]["requirement"]:
        offenders.append("assurance/pins.json:engineering_assurance.requirement")
    if FORBIDDEN_REGISTRY in pins["engineering_assurance"]["module_install"]:
        offenders.append("assurance/pins.json:engineering_assurance.module_install")
    return offenders


def build_report() -> dict[str, Any]:
    pins = json.loads(PINS_PATH.read_text(encoding="utf-8"))
    observed = [
        {"component": "quire-cli", "version": observe_quire()},
        {"component": "quoin", "version": observe_semver(["quoin", "--version"])},
        {"component": "ix-flow", "version": observe_semver(["ix-flow", "--version"])},
        {"component": "engineering-assurance", "version": observe_semver([cli_path(), "--version"])},
    ]
    result, _, _ = classify(observed)
    mismatches = declared_version_mismatches(pins, observed)
    offenders = mirror_references(pins)
    gate_satisfied = result["gate_satisfied"] and not mismatches and not offenders
    return {
        "schemaVersion": "tl-rewrite.shared-pin-report/v3",
        "installed_binary": binary_attestation(),
        "matrix_version": result["matrix_version"],
        "human_acceptance_recorded": result["human_acceptance_recorded"],
        "acceptance_recorded_here": False,
        "components_classified": len(result["components"]),
        "versions_compatible": result["versions_compatible"],
        "pin_mismatches": mismatches,
        "mirror_references": offenders,
        "gate_satisfied": gate_satisfied,
        "accepted": gate_satisfied,
        "components": result["components"],
    }


def main(argv: list[str]) -> int:
    as_json = argv[1:] == ["--json"]
    if argv[1:] and not as_json:
        print("usage: check_shared_pins.py [--json]", file=sys.stderr)
        return 2
    try:
        report = build_report()
    except (PinError, OSError, KeyError, TypeError) as error:
        print(str(error), file=sys.stderr)
        return 2
    if as_json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(f"Engineering Assurance binary SHA256: {report['installed_binary']['sha256']}")
        for item in report["components"]:
            print(f"{item['component']}: {item['observed']} -> {item['verdict']} ({item['reason']})")
        for mismatch in report["pin_mismatches"]:
            print(f"declared release mismatch: {mismatch}", file=sys.stderr)
        for offender in report["mirror_references"]:
            print(f"mirror registry reference: {offender}", file=sys.stderr)
        print(
            "shared pins accepted" if report["accepted"] else "shared pins NOT accepted",
            file=sys.stderr if not report["accepted"] else sys.stdout,
        )
    return 0 if report["accepted"] else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
