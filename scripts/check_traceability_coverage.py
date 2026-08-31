#!/usr/bin/env python3
"""Require complete traceability, status, and verification-reference coverage."""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
REFERENCE = re.compile(r"\b(?:TC|SUITE)-[0-9]{3}\b")
ALLOWED_DIAGNOSTICS = {
    "archetype-matches-nothing": ("inspection", "Inspections"),
    "catch-all-universal": ("criteria", "coverage.specific_shaped"),
    "status-column-matches-nothing": (
        "functional-coverage",
        "configured status column 'Status'",
    ),
}


def validate_report(report: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    totals = report.get("totals")
    if not isinstance(totals, dict):
        return ["coverage report has no totals object"]
    if totals.get("backed") != totals.get("total"):
        errors.append(f"coverage total is incomplete: {totals.get('backed')}/{totals.get('total')} backed")
    groups = report.get("groups")
    if not isinstance(groups, list) or not groups:
        errors.append("coverage report has no document groups")
    else:
        for group in groups:
            if not isinstance(group, dict) or group.get("backed") != group.get("total"):
                errors.append(f"coverage document group is incomplete: {group!r}")
    census = report.get("binding_census")
    if not isinstance(census, list) or not census:
        errors.append("coverage report has no binding census")
    else:
        for item in census:
            if not isinstance(item, dict):
                errors.append("coverage report contains malformed binding census")
                continue
            candidates = item.get("candidates")
            if item.get("tagged") != candidates or item.get("bound") != candidates:
                errors.append(f"{item.get('language', '<unknown>')} binding census is incomplete")
    for field in ("unbacked_rows", "status_lies", "untracked_symbols"):
        findings = report.get(field)
        if not isinstance(findings, list):
            errors.append(f"coverage report has no {field} list")
        elif findings:
            errors.append(f"coverage report contains {len(findings)} {field}")
    diagnostics = report.get("diagnostics")
    if not isinstance(diagnostics, list):
        errors.append("coverage report has no diagnostics list")
    else:
        for diagnostic in diagnostics:
            reason = diagnostic.get("reason") if isinstance(diagnostic, dict) else None
            expected = ALLOWED_DIAGNOSTICS.get(reason)
            message = diagnostic.get("message", "") if isinstance(diagnostic, dict) else ""
            declaration = diagnostic.get("declaration") if isinstance(diagnostic, dict) else None
            if expected is None:
                errors.append(f"coverage report contains blocking diagnostic {reason!r}")
            elif declaration != expected[0] or (
                expected[1] not in message
                and diagnostic.get("value") != expected[1]
            ):
                errors.append(f"coverage report contains changed diagnostic {reason!r}")
    return errors


def validate_matrix_statuses(path: Path) -> list[str]:
    errors: list[str] = []
    section = ""
    header: list[str] | None = None
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if line.startswith("## "):
            section = line[3:].strip()
            header = None
            continue
        if not line.startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if header is None:
            header = cells
            continue
        if all(set(cell) <= {"-", ":"} for cell in cells):
            continue
        status_name = "Status" if "Status" in header else "Coverage Status"
        if status_name not in header or len(cells) != len(header):
            errors.append(f"{path}:{number} has no parseable status column")
            continue
        status_index = header.index(status_name)
        for index, cell in enumerate(cells):
            if index != status_index and not cell:
                errors.append(f"{path}:{number} has an empty {header[index]!r} cell")
        expected = "✅ implemented" if section == "Test Case Summary" else "✅ covered"
        if cells[status_index] != expected:
            errors.append(f"{path}:{number} does not carry required status {expected!r}")
    return errors


def validate_verification_references() -> list[str]:
    matrix = (ROOT / "spec" / "test-matrix.md").read_text(encoding="utf-8")
    suites = (ROOT / "spec" / "evidence" / "suites.md").read_text(encoding="utf-8")
    declared = set(REFERENCE.findall(matrix)) | set(REFERENCE.findall(suites))
    errors: list[str] = []
    requirement_ids: set[str] = set()
    for path in sorted((ROOT / "spec" / "requirements").glob("*.md")):
        identity = re.search(r"^id:\s*((?:FR|NFR|StR)-[0-9]{3})\s*$", path.read_text(encoding="utf-8"), re.MULTILINE)
        if identity:
            requirement_ids.add(identity.group(1))
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            if not line.startswith("|") or not re.search(r"-(?:AC|VC)-[0-9]+\s*\|", line):
                continue
            targets = REFERENCE.findall(line)
            if not targets:
                errors.append(f"{path}:{number} declares no catalogued verification target")
            for target in targets:
                if target not in declared:
                    errors.append(f"{path}:{number} references nonexistent verification target {target}")
    matrix_ids = {
        cells[0]
        for line in matrix.splitlines()
        if line.startswith("|")
        for cells in [[cell.strip() for cell in line.strip().strip("|").split("|")]]
        if cells and re.fullmatch(r"(?:FR|NFR|StR)-[0-9]{3}", cells[0])
    }
    if matrix_ids != requirement_ids:
        errors.append(
            f"requirement matrix census drift: missing={sorted(requirement_ids - matrix_ids)}, "
            f"extra={sorted(matrix_ids - requirement_ids)}"
        )
    return errors


def main() -> int:
    result = subprocess.run(
        ["quire", "coverage", "--scope", ".", "--strict", "--json"],
        check=False,
        capture_output=True,
        text=True,
    )
    sys.stderr.write(result.stderr)
    if result.returncode != 0:
        return result.returncode
    try:
        report = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        print(f"quire emitted invalid coverage JSON: {error}", file=sys.stderr)
        return 2
    errors = validate_report(report)
    errors.extend(validate_matrix_statuses(ROOT / "spec" / "test-matrix.md"))
    errors.extend(validate_verification_references())
    for error in errors:
        print(error, file=sys.stderr)
    if errors:
        return 1
    totals = report["totals"]
    print(f"strict traceability coverage is complete: {totals['backed']}/{totals['total']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
