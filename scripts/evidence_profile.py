#!/usr/bin/env python3
"""Resolve retained evidence qualification profiles without silent downgrades."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
QUALIFICATION_V2 = "tl-rewrite.evidence-qualification/v2"
RETRACTIONS = ROOT / "evidence" / "RETRACTIONS.json"


def retracted_records() -> set[str]:
    value = json.loads(RETRACTIONS.read_text(encoding="utf-8"))
    if value.get("schemaVersion") != "tl-rewrite.evidence-retractions/v1":
        raise ValueError("evidence retraction registry has an unknown schema")
    records = value.get("records")
    if not isinstance(records, dict) or not all(
        isinstance(item, dict) and isinstance(item.get("reason"), str) and item["reason"]
        for item in records.values()
    ):
        raise ValueError("evidence retraction registry has a malformed record map")
    return set(records)


def resolve_profile(evidence_dir: Path) -> str:
    if evidence_dir.name in retracted_records():
        return "retracted"
    collection_input = json.loads(
        (evidence_dir / "collection-input.json").read_text(encoding="utf-8")
    )
    profile = collection_input.get("qualificationProfile")
    revision = (evidence_dir / "source-revision.txt").read_text(encoding="utf-8").strip()
    source_builder = subprocess.run(
        ["/usr/bin/git", "show", f"{revision}:scripts/build_evidence_envelope.py"],
        cwd=ROOT, check=True, capture_output=True,
    ).stdout
    source_requires_v2 = QUALIFICATION_V2.encode() in source_builder
    if profile == QUALIFICATION_V2:
        return "v2"
    if profile is None and not source_requires_v2:
        return "inconclusive"
    if profile is None:
        raise ValueError("v2-era evidence omits qualificationProfile")
    raise ValueError(f"unrecognized evidence qualification profile: {profile!r}")
