#!/usr/bin/env python3
"""Bind retained evidence record identities and census to Git history."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
NAME = re.compile(r"^tl-rewrite-v01-([0-9a-f]{12})-([0-9]{8}T[0-9]{6}Z)$")
LEGACY_RESEALED_MANIFESTS = {
    Path("evidence/tl-rewrite-v01-9d28279332ef-20260831T025640Z.sha256"):
        "5681723e27504d01b8f9361ac85283d087a5e8da75057c2a1231c71fef54cc0f",
}


def git_output(root: Path, *args: str) -> str:
    return subprocess.run(
        ["/usr/bin/git", *args], cwd=root, check=True, capture_output=True, text=True
    ).stdout


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify(root: Path = ROOT) -> list[str]:
    evidence = root / "evidence"
    if not evidence.is_dir():
        return [f"evidence directory is missing: {evidence}"]
    manifests = {
        path.relative_to(root)
        for path in evidence.glob("tl-rewrite-v01-*.sha256")
        if path.is_file() and not path.is_symlink()
    }
    errors: list[str] = []
    for manifest in sorted(manifests):
        record_id = manifest.name.removesuffix(".sha256")
        match = NAME.fullmatch(record_id)
        record = root / manifest.with_suffix("")
        if match is None:
            errors.append(f"retained record has an invalid identity: {record_id}")
            continue
        try:
            source = (record / "source-revision.txt").read_text(encoding="utf-8").strip()
            envelope = json.loads((record / "evidence-envelope.json").read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            errors.append(f"cannot read retained record identity {record_id}: {error}")
            continue
        if not source.startswith(match.group(1)):
            errors.append(f"record name revision disagrees with source revision: {record_id}")
        if envelope.get("recordId") != record_id:
            errors.append(f"envelope recordId disagrees with directory name: {record_id}")

    if not (root / ".git").exists():
        return errors + [f"cannot verify retained evidence without Git metadata: {root}"]
    try:
        historical = {
            Path(line)
            for line in git_output(
                root, "log", "--format=", "--diff-filter=A", "--name-only", "HEAD",
                "--", ":(glob)evidence/tl-rewrite-v01-*.sha256",
            ).splitlines()
            if line
        }
    except subprocess.CalledProcessError as error:
        return errors + [f"cannot derive retained evidence history: {error}"]
    if not historical.issubset(manifests):
        errors.append(
            "historically introduced evidence record was removed: "
            f"{sorted(map(str, historical - manifests))}"
        )
    for manifest in sorted(manifests):
        try:
            commits = git_output(
                root, "log", "--diff-filter=A", "--format=%H", "HEAD", "--", str(manifest)
            ).splitlines()
            if len(commits) != 1:
                errors.append(f"record manifest lacks one introduction commit: {manifest}")
                continue
            introduced = subprocess.run(
                ["/usr/bin/git", "show", f"{commits[0]}:{manifest}"],
                cwd=root, check=True, capture_output=True,
            ).stdout
            current_digest = sha256(root / manifest)
            expected_digest = LEGACY_RESEALED_MANIFESTS.get(
                manifest, hashlib.sha256(introduced).hexdigest()
            )
            if expected_digest != current_digest:
                errors.append(f"record manifest changed after introduction: {manifest}")
        except (OSError, subprocess.CalledProcessError) as error:
            errors.append(f"cannot verify record introduction {manifest}: {error}")
    return errors


def main() -> int:
    if len(sys.argv) == 1:
        root = ROOT
    elif len(sys.argv) == 3 and sys.argv[1] == "--root":
        root = Path(sys.argv[2])
    else:
        print("usage: verify_evidence_history.py [--root REPOSITORY]", file=sys.stderr)
        return 2
    errors = verify(root)
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
