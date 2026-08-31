#!/usr/bin/env python3
"""Write the post-envelope validation summary for a retained collection."""

from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path

from build_evidence_envelope import classify_result


CHECKS = (
    "make-ci",
    "make-spec",
    "quire-coverage",
    "rustdoc",
    "default-dependencies",
    "corpus-integrity",
    "diff-integrity",
    "input-schema",
    "manifest-schema",
    "pgm01-schema",
    "pgm01-validator",
    "sealed-pgm01-schema",
    "sealed-pgm01-validator",
)
CONTRADICTION = re.compile(
    r"test result: FAILED|Error [0-9]+ \(ignored\)|\b[1-9][0-9]* ignored\b"
)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def summary(evidence_dir: Path) -> dict[str, object]:
    outcomes = []
    observed = {
        path.name[: -len(".status.txt")]
        for path in evidence_dir.glob("*.status.txt")
        if path.is_file()
    }
    names = list(CHECKS) + sorted(observed - set(CHECKS))
    for name in names:
        status_path = evidence_dir / f"{name}.status.txt"
        if not status_path.exists():
            outcomes.append({"name": name, "status": "inconclusive", "exitCode": None})
            continue
        exit_code = int(status_path.read_text(encoding="utf-8").strip())
        skipped = exit_code == 125
        stderr_path = evidence_dir / f"{name}.stderr"
        validator_error = (
            exit_code == 0
            and
            name in {"pgm01-validator", "sealed-pgm01-validator"}
            and stderr_path.exists()
            and bool(stderr_path.read_text(encoding="utf-8").strip())
        )
        output_contradiction = any(
            path.exists()
            and CONTRADICTION.search(path.read_text(encoding="utf-8", errors="replace"))
            for path in (evidence_dir / f"{name}.stdout", evidence_dir / f"{name}.stderr")
        )
        outcomes.append(
            {
                "name": name,
                "status": (
                    "skipped-unavailable"
                    if skipped
                    else "failed"
                    if validator_error or output_contradiction
                    else "passed"
                    if exit_code == 0
                    else "failed"
                ),
                "exitCode": exit_code,
            }
        )
    statuses = {item["status"] for item in outcomes}
    if "failed" in statuses:
        overall = "failed"
    elif "skipped-unavailable" in statuses or "inconclusive" in statuses:
        overall = "inconclusive"
    else:
        overall = "passed"
    envelope = evidence_dir / "evidence-envelope.json"
    return {
        "schemaVersion": "tl-rewrite.collection-summary/v1",
        "overallStatus": overall,
        "finalEnvelopeSha256": sha256(envelope),
        "finalEnvelopeValidated": all(
            item["status"] == "passed"
            for item in outcomes
            if item["name"].startswith("sealed-")
        ),
        "outcomes": outcomes,
    }


def expected_envelope_result(evidence_dir: Path, value: dict[str, object]) -> dict[str, str]:
    outcomes = value["outcomes"]
    assert isinstance(outcomes, list)
    sealed = [item for item in outcomes if item["name"].startswith("sealed-")]
    sealed_files_present = all(
        (evidence_dir / f"{item['name']}.status.txt").is_file() for item in sealed
    )
    if not sealed_files_present:
        return {
            "status": "inconclusive",
            "summary": "legacy record lacks sealed PGM-01 outcomes; human gates remain pending",
        }
    phase = (
        "sealed-failed"
        if value["overallStatus"] == "failed"
        or any(item["status"] != "passed" for item in sealed)
        else "final"
    )
    status, result_summary = classify_result(phase, outcomes)
    return {"status": status, "summary": result_summary}


def validate_envelope_result(evidence_dir: Path, value: dict[str, object]) -> list[str]:
    try:
        envelope = json.loads((evidence_dir / "evidence-envelope.json").read_text(encoding="utf-8"))
        actual = envelope["result"]
        observed = {"status": actual["status"], "summary": actual["summary"]}
    except (KeyError, OSError, json.JSONDecodeError, TypeError) as error:
        return [f"cannot derive retained envelope result: {error}"]
    expected = expected_envelope_result(evidence_dir, value)
    return [] if observed == expected else [f"envelope result disagrees with retained outcomes: {evidence_dir}"]


def main() -> int:
    check = len(sys.argv) == 3 and sys.argv[1] == "--check"
    write = len(sys.argv) == 2 and sys.argv[1] != "--check"
    if not check and not write:
        print("usage: finalize_collection.py [--check] EVIDENCE_DIR", file=sys.stderr)
        return 2
    evidence_dir = Path(sys.argv[2] if check else sys.argv[1])
    try:
        value = summary(evidence_dir)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"cannot derive retained collection summary: {error}", file=sys.stderr)
        return 2
    envelope_errors = validate_envelope_result(evidence_dir, value)
    if envelope_errors:
        for error in envelope_errors:
            print(error, file=sys.stderr)
        return 1
    summary_path = evidence_dir / "collection-summary.json"
    if check:
        try:
            actual = json.loads(summary_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            print(f"cannot read retained summary: {error}", file=sys.stderr)
            return 2
        if actual != value:
            print(f"retained summary disagrees with status files: {evidence_dir}", file=sys.stderr)
            return 1
        return 0
    summary_path.write_text(
        json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
