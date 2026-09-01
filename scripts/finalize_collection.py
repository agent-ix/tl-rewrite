#!/usr/bin/env python3
"""Write the post-envelope validation summary for a retained collection."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

from build_evidence_envelope import classify_result
from evidence_profile import QUALIFICATION_V2, resolve_profile
import tool_identity


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
TEST_SUCCESS = re.compile(
    r"^test result: ok\. ([1-9][0-9]*) passed; 0 failed; 0 ignored", re.MULTILINE
)
CORROBORATED = "passed"
REJECTED = "failed"
NOT_APPLICABLE = "inconclusive"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_revision(evidence_dir: Path) -> str:
    return (evidence_dir / "source-revision.txt").read_text(encoding="utf-8").strip()


def source_text(evidence_dir: Path, relative: str) -> str | None:
    revision = source_revision(evidence_dir)
    root = Path(__file__).resolve().parent.parent
    subprocess.run(
        ["/usr/bin/git", "cat-file", "-e", f"{revision}^{{commit}}"],
        cwd=root, check=True, capture_output=True,
    )
    paths = subprocess.run(
        ["/usr/bin/git", "ls-tree", "-r", "--name-only", revision, "--", relative],
        cwd=root, check=True, capture_output=True, text=True,
    ).stdout.splitlines()
    if relative not in paths:
        return None
    return subprocess.run(
        ["/usr/bin/git", "show", f"{revision}:{relative}"],
        cwd=root, check=True, capture_output=True, text=True,
    ).stdout


def expected_test_markers(evidence_dir: Path) -> tuple[str, ...]:
    revision = source_revision(evidence_dir)
    paths = subprocess.run(
        ["/usr/bin/git", "ls-tree", "-r", "--name-only", revision, "--", "src/lib.rs", "tests"],
        cwd=Path(__file__).resolve().parent.parent,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.splitlines()
    tests = sorted(
        path for path in paths
        if Path(path).parent == Path("tests") and Path(path).suffix == ".rs"
    )
    if "src/lib.rs" not in paths or not tests:
        raise ValueError("cannot derive the Rust test-suite census from the source revision")
    return ("Running unittests src/lib.rs",) + tuple(f"Running {path}" for path in tests)


def positive_test_census(evidence_dir: Path, output: str, repetitions: int) -> bool:
    markers = expected_test_markers(evidence_dir)
    revision = source_revision(evidence_dir)
    test_count = 0
    for relative in subprocess.run(
        ["/usr/bin/git", "ls-tree", "-r", "--name-only", revision, "--", "src", "tests"],
        cwd=Path(__file__).resolve().parent.parent, check=True,
        capture_output=True, text=True,
    ).stdout.splitlines():
        if not relative.endswith(".rs"):
            continue
        source = subprocess.run(
            ["/usr/bin/git", "show", f"{revision}:{relative}"],
            cwd=Path(__file__).resolve().parent.parent, check=True,
            capture_output=True, text=True,
        ).stdout
        test_count += len(re.findall(r"(?m)^\s*#\[test\]\s*$", source))
    passed = [int(value) for value in TEST_SUCCESS.findall(output)]
    return (
        len(passed) == len(markers) * repetitions
        and test_count > 0
        and sum(passed) == test_count * repetitions
        and all(output.count(marker) >= repetitions for marker in markers)
    )


def source_paths(evidence_dir: Path, *roots: str) -> list[str]:
    return subprocess.run(
        ["/usr/bin/git", "ls-tree", "-r", "--name-only", source_revision(evidence_dir), "--", *roots],
        cwd=Path(__file__).resolve().parent.parent, check=True, capture_output=True, text=True,
    ).stdout.splitlines()


def expected_policy_test_count(evidence_dir: Path) -> int:
    return sum(
        Path(path).name.startswith("test_") and Path(path).suffix == ".py"
        for path in source_paths(evidence_dir, "scripts")
    )


def expected_trace_total(evidence_dir: Path) -> int:
    identities: set[str] = set()
    for relative in source_paths(evidence_dir, "spec"):
        if not relative.endswith(".md"):
            continue
        text = source_text(evidence_dir, relative)
        assert text is not None
        identities.update(re.findall(r"(?m)^\|\s*((?:FR|NFR|StR)-[0-9]+-(?:AC|VC)-[0-9]+)\s*\|", text))
        identities.update(re.findall(r"(?m)^\|\s*((?:TC|SUITE)-[0-9]+)\s*\|", text))
    return len(identities)


def expected_candidate_target_count(makefile: str) -> int:
    match = re.search(r"(?m)^ci-for-evidence:\s+(.+)$", makefile)
    return len(match.group(1).split()) if match else 0


def positive_output(evidence_dir: Path, name: str) -> str:
    output = "\n".join(
        path.read_text(encoding="utf-8", errors="replace")
        for path in (evidence_dir / f"{name}.stdout", evidence_dir / f"{name}.stderr")
        if path.exists()
    )
    if name == "diff-integrity":
        checker = source_text(evidence_dir, "scripts/check_diff_integrity.py")
        if checker is None:
            return NOT_APPLICABLE
        return CORROBORATED if (
            f"diff-integrity gate passed for origin/main...{source_revision(evidence_dir)}" in output
        ) else REJECTED
    if name == "make-ci":
        makefile = source_text(evidence_dir, "Makefile")
        if makefile is None:
            return NOT_APPLICABLE
        known_signatures = (
            "qualified-tool-identities gate passed", "fmt-check gate passed",
            "lint gate passed", "Rust test gate passed", "corpus-integrity gate passed",
            "deny gate passed", "audit-unsafe gate passed", "evidence-tool gate passed",
            "spec gate passed", "msrv gate passed", "rustdoc gate passed",
        )
        if any(signature not in makefile for signature in known_signatures):
            return NOT_APPLICABLE
        target_count = expected_candidate_target_count(makefile)
        policy_count = expected_policy_test_count(evidence_dir)
        trace_total = expected_trace_total(evidence_dir)
        passed = (
            positive_test_census(evidence_dir, output, 2)
            and target_count > 0
            and f"all {target_count} mandatory local-CI targets propagate failures" in output
            and policy_count > 0
            and f"all {policy_count} evidence-policy behavior tests passed" in output
            and trace_total > 0
            and f"strict traceability coverage is complete: {trace_total}/{trace_total}" in output
            and "licenses ok" in output
            and "sources ok" in output
            and re.search(r"advisories ok at [0-9a-f]{40}", output) is not None
            and "Generated " in output and "/doc/tl_rewrite/index.html" in output
            and all(signature in output for signature in known_signatures)
        )
        return CORROBORATED if passed else REJECTED
    if name == "msrv":
        return CORROBORATED if positive_test_census(evidence_dir, output, 1) else REJECTED
    if name == "rustdoc":
        checker = source_text(evidence_dir, "scripts/check_rustdoc.sh")
        if checker is None:
            return NOT_APPLICABLE
        return CORROBORATED if (
            "Generated " in output and "/doc/tl_rewrite/index.html" in output
            and re.search(r"observed rustdoc index SHA-256 [0-9a-f]{64}", output) is not None
        ) else REJECTED
    if name == "quire-coverage":
        total = expected_trace_total(evidence_dir)
        return CORROBORATED if total > 0 and f"Coverage: {total}/{total}" in output else REJECTED
    if name == "make-spec":
        total = expected_trace_total(evidence_dir)
        return CORROBORATED if total > 0 and (
            f"strict traceability coverage is complete: {total}/{total}" in output
        ) else REJECTED
    if name == "default-dependencies":
        manifest = source_text(evidence_dir, "Cargo.toml")
        if manifest is None:
            return NOT_APPLICABLE
        revisions = re.findall(r'git\s*=\s*"[^"]+"\s*,\s*rev\s*=\s*"([0-9a-f]{40})"', manifest)
        return CORROBORATED if (
            "tl-rewrite v0.1.0" in output
            and len(revisions) >= 2
            and all(revision[:8] in output for revision in revisions)
        ) else REJECTED
    if name == "corpus-integrity":
        checker = source_text(evidence_dir, "scripts/check_corpus.py")
        if checker is None or "WEST corpus checksum census passed" not in checker:
            return NOT_APPLICABLE
        return CORROBORATED if (
            "WEST corpus checksum census passed" in output
        ) else REJECTED
    if name in {
        "input-schema", "manifest-schema", "pgm01-schema", "pgm01-validator",
        "sealed-pgm01-schema", "sealed-pgm01-validator",
    }:
        return CORROBORATED if re.search(
            r'"errors"\s*:\s*\[\]\s*,?\s*"valid"\s*:\s*true', output
        ) else REJECTED
    return REJECTED


def summary(evidence_dir: Path) -> dict[str, object]:
    profile = resolve_profile(evidence_dir)
    if profile == "retracted":
        raise ValueError("retracted evidence cannot produce an active qualification summary")
    collection_input = json.loads(
        (evidence_dir / "collection-input.json").read_text(encoding="utf-8")
    )
    declared_v2 = collection_input.get("qualificationProfile") == QUALIFICATION_V2
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
        corroboration = (
            positive_output(evidence_dir, name)
            if exit_code == 0 and declared_v2
            else None
        )
        outcomes.append(
            {
                "name": name,
                "status": (
                    "skipped-unavailable"
                    if skipped
                    else "failed"
                    if validator_error or output_contradiction or corroboration == REJECTED
                    else "inconclusive"
                    if corroboration == NOT_APPLICABLE
                    else "passed"
                    if exit_code == 0
                    else "failed"
                ),
                "exitCode": exit_code,
            }
        )
    statuses = {item["status"] for item in outcomes}
    if profile == "inconclusive":
        statuses.add("inconclusive")
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


def validate_tool_identity(evidence_dir: Path) -> list[str]:
    revision = source_revision(evidence_dir)
    root = Path(__file__).resolve().parent.parent
    lock_result = subprocess.run(
        ["/usr/bin/git", "show", f"{revision}:tools.lock"], cwd=root,
        check=False, capture_output=True,
    )
    if lock_result.returncode != 0:
        return []
    try:
        expected = tool_identity.validate_lock(json.loads(lock_result.stdout))
        collection_input = json.loads(
            (evidence_dir / "collection-input.json").read_text(encoding="utf-8")
        )
        observed = {"tools": collection_input["tools"]["identities"]}
        if "runtimeIdentities" in expected:
            observed["runtimeIdentities"] = collection_input["tools"]["runtimeIdentities"]
    except (KeyError, OSError, ValueError, json.JSONDecodeError) as error:
        return [f"cannot rederive retained tool identities: {error}"]
    return [] if observed == expected else [
        f"retained tool identities disagree with source tools.lock: {evidence_dir}"
    ]


def validate_content_digests(evidence_dir: Path) -> list[str]:
    try:
        envelope = json.loads((evidence_dir / "evidence-envelope.json").read_text())
        input_digest = envelope["inputs"][0]["contentDigest"]["value"]
        output_digest = envelope["outputs"][0]["contentDigest"]["value"]
    except (IndexError, KeyError, OSError, TypeError, json.JSONDecodeError) as error:
        return [f"cannot read retained envelope content digests: {error}"]
    errors = []
    if input_digest != sha256(evidence_dir / "collection-input.json"):
        errors.append(f"envelope input content digest disagrees: {evidence_dir}")
    if output_digest != sha256(evidence_dir / "evidence-manifest.json"):
        errors.append(f"envelope output content digest disagrees: {evidence_dir}")
    return errors


def main() -> int:
    check = len(sys.argv) == 3 and sys.argv[1] == "--check"
    write = len(sys.argv) == 2 and sys.argv[1] != "--check"
    if not check and not write:
        print("usage: finalize_collection.py [--check] EVIDENCE_DIR", file=sys.stderr)
        return 2
    evidence_dir = Path(sys.argv[2] if check else sys.argv[1])
    try:
        profile = resolve_profile(evidence_dir)
    except (OSError, ValueError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        print(f"cannot resolve evidence qualification profile: {error}", file=sys.stderr)
        return 2
    if profile == "retracted":
        if not check:
            print(f"refusing to rewrite explicitly retracted evidence: {evidence_dir}", file=sys.stderr)
            return 2
        print(f"retained evidence is explicitly retracted: {evidence_dir}")
        return 0
    try:
        value = summary(evidence_dir)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"cannot derive retained collection summary: {error}", file=sys.stderr)
        return 2
    envelope_errors = validate_envelope_result(evidence_dir, value)
    envelope_errors.extend(validate_tool_identity(evidence_dir))
    envelope_errors.extend(validate_content_digests(evidence_dir))
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
