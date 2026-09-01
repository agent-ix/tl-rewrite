#!/usr/bin/env python3
"""Bind the assurance claim to one passing, reproducible retained record."""

from __future__ import annotations

import ast
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

import build_evidence_envelope as builder
from evidence_profile import resolve_profile
import tool_identity


ROOT = Path(__file__).resolve().parent.parent


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_bytes(revision: str, path: Path) -> bytes:
    relative = path.relative_to(ROOT)
    return subprocess.run(
        ["/usr/bin/git", "show", f"{revision}:{relative}"],
        cwd=ROOT,
        check=True,
        capture_output=True,
    ).stdout


def historical_fixed_parameter_paths(source_builder: bytes) -> set[str]:
    """Read the historical builder's fixed parameter tuple without executing it."""
    module = ast.parse(source_builder.decode("utf-8"))
    bindings: dict[str, Path] = {}

    def path_value(node: ast.AST) -> Path | None:
        if isinstance(node, ast.Name):
            if node.id == "ROOT":
                return Path()
            return bindings.get(node.id)
        if isinstance(node, ast.Constant) and isinstance(node.value, str):
            return Path(node.value)
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div):
            left = path_value(node.left)
            right = path_value(node.right)
            if left is not None and right is not None:
                return left / right
        return None

    parameter_function: ast.FunctionDef | None = None
    for statement in module.body:
        if isinstance(statement, (ast.FunctionDef, ast.AsyncFunctionDef)):
            if statement.name == "parameter_paths" and isinstance(statement, ast.FunctionDef):
                parameter_function = statement
            continue
        if (
            isinstance(statement, ast.Assign)
            and len(statement.targets) == 1
            and isinstance(statement.targets[0], ast.Name)
        ):
            value = path_value(statement.value)
            if value is not None:
                bindings[statement.targets[0].id] = value
    if parameter_function is None:
        raise ValueError("historical builder has no parameter_paths function")
    fixed_value: ast.AST | None = None
    for statement in parameter_function.body:
        if (
            isinstance(statement, ast.Assign)
            and len(statement.targets) == 1
            and isinstance(statement.targets[0], ast.Name)
            and statement.targets[0].id == "fixed"
        ):
            fixed_value = statement.value
            break
    if not isinstance(fixed_value, (ast.Tuple, ast.List)):
        raise ValueError("historical builder has no literal fixed parameter census")
    paths: set[str] = set()
    for item in fixed_value.elts:
        value = path_value(item)
        if value is None or value.is_absolute() or value == Path():
            raise ValueError("historical builder has an unsupported fixed parameter expression")
        paths.add(str(value))
    return paths


def historical_parameters_digest(revision: str) -> str:
    """Recreate the builder's parameter set from the retained source tree."""
    tree = set(
        subprocess.run(
            ["/usr/bin/git", "ls-tree", "-r", "--name-only", revision],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
    )
    source_builder = git_bytes(revision, builder.BUILDER)
    fixed = historical_fixed_parameter_paths(source_builder)
    scripts = {
        path
        for path in tree
        if path.startswith("scripts/") and Path(path).suffix in {".py", ".sh"}
    }
    paths = sorted(fixed | scripts)
    missing = [path for path in paths if path not in tree]
    if missing:
        raise OSError(f"source revision lacks parameter paths: {', '.join(missing)}")
    state = hashlib.sha256()
    for relative in paths:
        state.update(relative.encode())
        state.update(b"\0")
        state.update(git_bytes(revision, ROOT / relative))
        state.update(b"\0")
    return state.hexdigest()


def main() -> int:
    assurance_path = Path(sys.argv[1]) if len(sys.argv) == 2 else ROOT / "spec" / "assurance" / "AA-001.md"
    if len(sys.argv) > 2:
        print("usage: check_assurance_anchor.py [ASSURANCE_PATH]", file=sys.stderr)
        return 2
    try:
        assurance = assurance_path.read_text(encoding="utf-8")
    except OSError as error:
        print(f"cannot read assurance argument: {error}", file=sys.stderr)
        return 2
    record_matches = re.findall(r"`(evidence/tl-rewrite-v01-[^`]+)`", assurance)
    record_match = re.search(r"`(evidence/tl-rewrite-v01-[^`]+)`", assurance)
    outer_match = re.search(r"checksum-manifest\s+SHA-256\s+`([0-9a-f]{64})`", assurance, re.DOTALL)
    envelope_match = re.search(r"final envelope SHA-256 is\s+`([0-9a-f]{64})`", assurance, re.DOTALL)
    count_match = re.search(r"All\s+([0-9]+)\s+collection and post-seal outcomes passed", assurance)
    if len(record_matches) != 1 or any(
        match is None for match in (record_match, outer_match, envelope_match, count_match)
    ):
        print("assurance argument does not identify one record, digests, and outcome count", file=sys.stderr)
        return 1
    assert record_match and outer_match and envelope_match and count_match
    record = record_match.group(1)
    record_dir = ROOT / record
    checksum = ROOT / f"{record}.sha256"
    try:
        outer_digest = sha256(checksum)
        summary = json.loads((record_dir / "collection-summary.json").read_text(encoding="utf-8"))
        envelope = json.loads((record_dir / "evidence-envelope.json").read_text(encoding="utf-8"))
        collection_input = json.loads((record_dir / "collection-input.json").read_text(encoding="utf-8"))
        source_revision = (record_dir / "source-revision.txt").read_text(encoding="utf-8").strip()
    except (OSError, json.JSONDecodeError) as error:
        print(f"assurance argument names an unreadable retained record: {error}", file=sys.stderr)
        return 1
    errors: list[str] = []
    if outer_digest != outer_match.group(1):
        errors.append("assurance argument does not bind its record's outer manifest digest")
    anchor = f"{outer_digest}  {record}.sha256"
    try:
        anchors = (ROOT / "evidence" / "ANCHORS").read_text(encoding="utf-8").splitlines()
    except OSError as error:
        print(f"cannot read evidence anchors: {error}", file=sys.stderr)
        return 2
    if anchor not in anchors:
        errors.append("assurance argument and evidence anchors disagree")
    outcomes = summary.get("outcomes")
    claimed_count = int(count_match.group(1))
    if summary.get("overallStatus") != "passed" or not isinstance(outcomes, list):
        errors.append("assurance argument does not name a passing retained record")
    elif len(outcomes) != claimed_count or any(item.get("status") != "passed" for item in outcomes):
        errors.append("assurance outcome-count claim disagrees with the retained summary")
    envelope_digest = sha256(record_dir / "evidence-envelope.json")
    if envelope_digest != envelope_match.group(1) or summary.get("finalEnvelopeSha256") != envelope_digest:
        errors.append("assurance final-envelope digest disagrees with the retained record")
    if envelope.get("provenance", {}).get("sourceRevision") != source_revision:
        errors.append("envelope provenance does not bind the retained source revision")
    if envelope.get("provenance", {}).get("candidateRevision") != source_revision:
        errors.append("envelope candidate revision does not bind the retained source revision")
    if envelope.get("provenance", {}).get("reviewers") != ["@kreneskyp"]:
        errors.append("envelope reviewer provenance is not the declared pending reviewer")
    if envelope.get("extensions", {}).get("dev.agent-ix.tl-rewrite", {}).get("reviewState") != "pending":
        errors.append("envelope review state is not pending human review")
    try:
        source_collector = git_bytes(source_revision, builder.COLLECTOR)
        profile = resolve_profile(record_dir)
        if profile == "v2":
            tools = collection_input["tools"]
            retained_path = (record_dir / "python-path.txt").read_text(encoding="utf-8").strip()
            retained_checkers = json.loads(
                (record_dir / "jsonschema-format-checkers.json").read_text(encoding="utf-8")
            )
            if tools.get("pythonPath") != retained_path or not retained_path.startswith("/"):
                errors.append("record does not retain its resolved Python interpreter path")
            if tools.get("jsonschemaFormatCheckers") != retained_checkers or "date-time" not in retained_checkers:
                errors.append("record does not retain an active date-time format checker")
            expected_parameters = historical_parameters_digest(source_revision)
            if envelope.get("parametersDigest", {}).get("value") != expected_parameters:
                errors.append("envelope parameters digest does not match the source revision")
            if envelope.get("producer", {}).get("executableDigest", {}).get("value") != hashlib.sha256(source_collector).hexdigest():
                errors.append("envelope executable digest does not match the source revision")
            lock_digest = hashlib.sha256(git_bytes(source_revision, ROOT / "Cargo.lock")).hexdigest()
            if envelope.get("environment", {}).get("dependenciesDigest", {}).get("value") != lock_digest:
                errors.append("envelope dependency digest does not match the source revision")
            source_tool_lock = json.loads(git_bytes(source_revision, builder.TOOLS_LOCK))
            expected_tools = tool_identity.validate_lock(source_tool_lock)
            observed_tools = {"tools": tools.get("identities")}
            if "runtimeIdentities" in expected_tools:
                observed_tools["runtimeIdentities"] = tools.get("runtimeIdentities")
            if observed_tools != expected_tools:
                errors.append("record tool identities do not match the source tool lock")
        elif profile == "retracted":
            errors.append("assured evidence is explicitly retracted")
        else:
            errors.append("assured evidence is not v2-qualified")
    except (KeyError, OSError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        errors.append(f"cannot rederive assured evidence identities: {error}")
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
