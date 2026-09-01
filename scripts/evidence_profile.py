#!/usr/bin/env python3
"""Resolve retained evidence qualification profiles without silent downgrades."""

from __future__ import annotations

import json
import hashlib
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
QUALIFICATION_V2 = "tl-rewrite.evidence-qualification/v2"
RETRACTIONS = ROOT / "evidence" / "RETRACTIONS.json"
RECORD_NAME = re.compile(r"^tl-rewrite-v01-([0-9a-f]{12})-[0-9]{8}T[0-9]{6}Z$")
LEGACY_REASON = "legacy record without the active qualification profile"


def retracted_records(root: Path = ROOT) -> set[str]:
    value = json.loads((root / "evidence" / "RETRACTIONS.json").read_text(encoding="utf-8"))
    if value.get("schemaVersion") != "tl-rewrite.evidence-retractions/v1":
        raise ValueError("evidence retraction registry has an unknown schema")
    records = value.get("records")
    if not isinstance(records, dict):
        raise ValueError("evidence retraction registry has a malformed record map")
    for record_id, item in records.items():
        match = RECORD_NAME.fullmatch(record_id)
        if match is None or not isinstance(item, dict):
            raise ValueError(f"evidence retraction registry has a malformed entry: {record_id}")
        required = {"manifestSha256", "reason", "retractedAt", "reviewer", "sourceRevision"}
        if not required.issubset(item) or set(item) - required - {"reseal"}:
            raise ValueError(f"evidence retraction entry has unknown or missing fields: {record_id}")
        revision = item["sourceRevision"]
        if not isinstance(revision, str) or len(revision) != 40 or not revision.startswith(match.group(1)):
            raise ValueError(f"evidence retraction source revision is malformed: {record_id}")
        if item["reason"] != LEGACY_REASON or item["reviewer"] != "@kreneskyp":
            raise ValueError(f"evidence retraction disposition is unsupported: {record_id}")
        if not isinstance(item["retractedAt"], str) or not item["retractedAt"].endswith("Z"):
            raise ValueError(f"evidence retraction date is malformed: {record_id}")
        record = root / "evidence" / record_id
        manifest = record.with_suffix(".sha256")
        try:
            observed_revision = (record / "source-revision.txt").read_text().strip()
            collection_input = json.loads((record / "collection-input.json").read_text())
            manifest_digest = hashlib.sha256(manifest.read_bytes()).hexdigest()
        except (OSError, json.JSONDecodeError) as error:
            raise ValueError(f"evidence retraction target is unreadable: {record_id}: {error}") from error
        if observed_revision != revision or collection_input.get("qualificationProfile") is not None:
            raise ValueError(f"evidence retraction reason is not established by its record: {record_id}")
        if item["manifestSha256"] != manifest_digest:
            raise ValueError(f"evidence retraction manifest digest disagrees: {record_id}")
        reseal = item.get("reseal")
        if reseal is not None and (
            not isinstance(reseal, dict)
            or set(reseal) != {"introducedManifestSha256", "reason"}
            or not isinstance(reseal["reason"], str)
            or not reseal["reason"]
            or not re.fullmatch(r"[0-9a-f]{64}", reseal["introducedManifestSha256"])
        ):
            raise ValueError(f"evidence retraction reseal disposition is malformed: {record_id}")
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
