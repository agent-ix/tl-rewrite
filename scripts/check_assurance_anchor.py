#!/usr/bin/env python3
"""Bind the assurance claim to one passing, reproducible retained record."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

import build_evidence_envelope as builder


ROOT = Path(__file__).resolve().parent.parent


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_bytes(revision: str, path: Path) -> bytes:
    relative = path.relative_to(ROOT)
    return subprocess.run(
        ["git", "show", f"{revision}:{relative}"],
        cwd=ROOT,
        check=True,
        capture_output=True,
    ).stdout


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
    record_match = re.search(r"`(evidence/tl-rewrite-v01-[^`]+)`", assurance)
    outer_match = re.search(r"checksum-manifest\s+SHA-256\s+`([0-9a-f]{64})`", assurance, re.DOTALL)
    envelope_match = re.search(r"final envelope SHA-256 is\s+`([0-9a-f]{64})`", assurance, re.DOTALL)
    count_match = re.search(r"All\s+([0-9]+)\s+collection and post-seal outcomes passed", assurance)
    if any(match is None for match in (record_match, outer_match, envelope_match, count_match)):
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
        if b"jsonschema-format-checkers.json" in source_collector:
            tools = collection_input["tools"]
            retained_path = (record_dir / "python-path.txt").read_text(encoding="utf-8").strip()
            retained_checkers = json.loads(
                (record_dir / "jsonschema-format-checkers.json").read_text(encoding="utf-8")
            )
            if tools.get("pythonPath") != retained_path or not retained_path.startswith("/"):
                errors.append("record does not retain its resolved Python interpreter path")
            if tools.get("jsonschemaFormatCheckers") != retained_checkers or "date-time" not in retained_checkers:
                errors.append("record does not retain an active date-time format checker")
            expected_parameters = builder.parameters_digest(
                lambda path: git_bytes(source_revision, path)
            )
            if envelope.get("parametersDigest", {}).get("value") != expected_parameters:
                errors.append("envelope parameters digest does not match the source revision")
            if envelope.get("producer", {}).get("executableDigest", {}).get("value") != hashlib.sha256(source_collector).hexdigest():
                errors.append("envelope executable digest does not match the source revision")
            lock_digest = hashlib.sha256(git_bytes(source_revision, ROOT / "Cargo.lock")).hexdigest()
            if envelope.get("environment", {}).get("dependenciesDigest", {}).get("value") != lock_digest:
                errors.append("envelope dependency digest does not match the source revision")
    except (KeyError, OSError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        errors.append(f"cannot rederive assured evidence identities: {error}")
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
