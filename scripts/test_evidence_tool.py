#!/usr/bin/env python3
"""Behavior tests for evidence outcome classification."""

from __future__ import annotations

import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location(
    "build_evidence_envelope", ROOT / "scripts" / "build_evidence_envelope.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
FINALIZER_SPEC = importlib.util.spec_from_file_location(
    "finalize_collection", ROOT / "scripts" / "finalize_collection.py"
)
assert FINALIZER_SPEC is not None and FINALIZER_SPEC.loader is not None
FINALIZER = importlib.util.module_from_spec(FINALIZER_SPEC)
FINALIZER_SPEC.loader.exec_module(FINALIZER)
VERIFIER_SPEC = importlib.util.spec_from_file_location(
    "verify_evidence_manifest", ROOT / "scripts" / "verify_evidence_manifest.py"
)
assert VERIFIER_SPEC is not None and VERIFIER_SPEC.loader is not None
VERIFIER = importlib.util.module_from_spec(VERIFIER_SPEC)
VERIFIER_SPEC.loader.exec_module(VERIFIER)
ADVISORY_SPEC = importlib.util.spec_from_file_location(
    "check_advisories", ROOT / "scripts" / "check_advisories.py"
)
assert ADVISORY_SPEC is not None and ADVISORY_SPEC.loader is not None
ADVISORY = importlib.util.module_from_spec(ADVISORY_SPEC)
ADVISORY_SPEC.loader.exec_module(ADVISORY)
ANCHOR_SPEC = importlib.util.spec_from_file_location(
    "check_assurance_anchor", ROOT / "scripts" / "check_assurance_anchor.py"
)
assert ANCHOR_SPEC is not None and ANCHOR_SPEC.loader is not None
ANCHOR = importlib.util.module_from_spec(ANCHOR_SPEC)
ANCHOR_SPEC.loader.exec_module(ANCHOR)
PROPAGATION_SPEC = importlib.util.spec_from_file_location(
    "check_failure_propagation", ROOT / "scripts" / "check_failure_propagation.py"
)
assert PROPAGATION_SPEC is not None and PROPAGATION_SPEC.loader is not None
PROPAGATION = importlib.util.module_from_spec(PROPAGATION_SPEC)
PROPAGATION_SPEC.loader.exec_module(PROPAGATION)


def policy_test_count() -> int:
    return sum(
        path.name.startswith("test_") and path.suffix == ".py"
        for path in (ROOT / "scripts").iterdir()
    )


def trace_total() -> int:
    identities: set[str] = set()
    for path in (ROOT / "spec").rglob("*.md"):
        text = path.read_text(encoding="utf-8")
        identities.update(
            re.findall(
                r"(?m)^\|\s*((?:FR|NFR|StR)-[0-9]+-(?:AC|VC)-[0-9]+)\s*\|",
                text,
            )
        )
        identities.update(re.findall(r"(?m)^\|\s*((?:TC|SUITE)-[0-9]+)\s*\|", text))
    return len(identities)


def healthy_test_output(repetitions: int) -> str:
    paths = sorted((ROOT / "tests").glob("*.rs"))
    markers = ["Running unittests src/lib.rs"] + [f"Running tests/{path.name}" for path in paths]
    counts = [
        sum(path.read_text(encoding="utf-8").count("#[test]") for path in (ROOT / "src").glob("*.rs"))
    ] + [path.read_text(encoding="utf-8").count("#[test]") for path in paths]
    assert all(count > 0 for count in counts)
    return "".join(
        "".join(
            f"{marker}\ntest result: ok. {count} passed; 0 failed; 0 ignored\n"
            for marker, count in zip(markers, counts, strict=True)
        )
        for _ in range(repetitions)
    )


def healthy_ci_output() -> str:
    traces = trace_total()
    return healthy_test_output(2) + (
        f"all {len(PROPAGATION.PROBE_TARGETS)} mandatory local-CI targets propagate failures\n"
        f"all {policy_test_count()} evidence-policy behavior tests passed\n"
        f"strict traceability coverage is complete: {traces}/{traces}\n"
        "licenses ok\nsources ok\n"
        f"advisories ok at {'b' * 40}\n"
        "Generated /tmp/doc/tl_rewrite/index.html\n"
        "fmt-check gate passed\n"
        "lint gate passed\n"
        "Rust test gate passed\n"
        "corpus-integrity gate passed\n"
        "deny gate passed\n"
        "audit-unsafe gate passed\n"
        "evidence-tool gate passed\n"
        "spec gate passed\n"
        "msrv gate passed\n"
        "rustdoc gate passed\n"
        "qualified-tool-identities gate passed\n"
        f"observed rustdoc index SHA-256 {'a' * 64}\n"
        "verify-evidence gate passed\n"
    )


def healthy_msrv_output() -> str:
    return healthy_test_output(1)


def assert_output_rejected(evidence_dir: Path, name: str, mutated: str) -> None:
    path = evidence_dir / f"{name}.stdout"
    original = path.read_text(encoding="utf-8")
    path.write_text(mutated, encoding="utf-8")
    assert FINALIZER.positive_output(evidence_dir, name) == FINALIZER.REJECTED, (
        f"{name} accepted a missing positive-output conjunct"
    )
    path.write_text(original, encoding="utf-8")


def main() -> int:
    if sys.flags.optimize or os.environ.get("PYTHONOPTIMIZE"):
        return 2
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        config = root / "deny.toml"
        config.write_text(
            '[advisories]\ndb-path = "$CARGO_HOME/tl-rewrite-advisory-db"\n',
            encoding="utf-8",
        )
        explicit = root / "explicit-cargo"
        assert ADVISORY.database_root(
            config, {"CARGO_HOME": str(explicit), "HOME": str(root / "ignored")}
        ) == explicit / "tl-rewrite-advisory-db"
        home = root / "qualified-home"
        assert ADVISORY.database_root(config, {"HOME": str(home)}) == (
            home / ".cargo" / "tl-rewrite-advisory-db"
        )
        actual_cargo_home = root / "actual-cargo"
        assert ADVISORY.database_root(
            environment={"CARGO_HOME": str(actual_cargo_home)}
        ) == actual_cargo_home / "tl-rewrite-advisory-db"
        database = actual_cargo_home / "tl-rewrite-advisory-db" / "checkout"
        database.mkdir(parents=True)
        subprocess.run(["/usr/bin/git", "init", "-q"], cwd=database, check=True)
        subprocess.run(
            ["/usr/bin/git", "remote", "add", "origin", ADVISORY.REPOSITORY + ".git"],
            cwd=database,
            check=True,
        )
        (database / "advisory.txt").write_text("reviewed\n", encoding="utf-8")
        subprocess.run(["/usr/bin/git", "add", "advisory.txt"], cwd=database, check=True)
        subprocess.run(
            [
                "/usr/bin/git", "-c", "user.name=Advisory Fixture", "-c",
                "user.email=advisory@example.invalid", "commit", "-qm", "fixture",
            ],
            cwd=database,
            check=True,
        )
        revision = subprocess.run(
            ["/usr/bin/git", "rev-parse", "HEAD"], cwd=database, check=True,
            capture_output=True, text=True,
        ).stdout.strip()
        assert ADVISORY.database_identity(actual_cargo_home / "tl-rewrite-advisory-db") == {
            "repository": ADVISORY.REPOSITORY,
            "revision": revision,
        }
    assured = ROOT / "evidence" / "tl-rewrite-v01-1b08a6c9e7bc-20260831T203039Z"
    assert VERIFIER.verify(assured) == [], (
        "manifest verifier rejected the collector's repository-relative checksum paths"
    )
    assert FINALIZER.validate_content_digests(assured) == []
    builder_source = (ROOT / "scripts" / "build_evidence_envelope.py").read_bytes()
    fixed_parameters = ANCHOR.historical_fixed_parameter_paths(builder_source)
    renamed_builder = builder_source.replace(b"TOOLS_LOCK", b"RENAMED_TOOL_LOCK").replace(
        b"EVIDENCE_RETRACTIONS", b"RENAMED_EVIDENCE_RETRACTIONS"
    )
    assert ANCHOR.historical_fixed_parameter_paths(renamed_builder) == fixed_parameters, (
        "historical parameter derivation still depends on builder constant names"
    )
    assert FINALIZER.resolve_profile(assured) == "inconclusive"
    assert FINALIZER.positive_output(assured, "make-ci") == FINALIZER.NOT_APPLICABLE
    assert FINALIZER.positive_output(assured, "rustdoc") == FINALIZER.NOT_APPLICABLE
    assurance_result = subprocess.run(
        [sys.executable, str(ROOT / "scripts" / "check_assurance_anchor.py")],
        check=False, capture_output=True, text=True,
    )
    assert assurance_result.returncode != 0 and "not v2-qualified" in assurance_result.stderr
    with tempfile.TemporaryDirectory() as directory:
        digest_fixture = Path(directory)
        for name in ("collection-input.json", "evidence-manifest.json", "evidence-envelope.json"):
            shutil.copy2(assured / name, digest_fixture / name)
        (digest_fixture / "collection-input.json").write_text("{}\n", encoding="utf-8")
        assert FINALIZER.validate_content_digests(digest_fixture), (
            "envelope input contentDigest was not rederived"
        )
    missing_assurance = subprocess.run(
        [
            sys.executable,
            str(ROOT / "scripts" / "check_assurance_anchor.py"),
            "/definitely/missing/tl-rewrite-assurance.md",
        ],
        check=False,
        capture_output=True,
    )
    assert missing_assurance.returncode != 0, "assurance gate accepted a missing argument"
    with tempfile.TemporaryDirectory() as directory:
        evidence_dir = Path(directory) / "record"
        evidence_dir.mkdir()
        verification_dir = evidence_dir
        locked_identities = FINALIZER.tool_identity.validate_lock(
            json.loads((ROOT / "tools.lock").read_text(encoding="utf-8"))
        )
        (evidence_dir / "collection-input.json").write_text(
            json.dumps({
                "qualificationProfile": "tl-rewrite.evidence-qualification/v2",
                "tools": {
                    "identities": locked_identities["tools"],
                    "runtimeIdentities": locked_identities["runtimeIdentities"],
                },
            }),
            encoding="utf-8",
        )
        revision = subprocess.run(
            ["/usr/bin/git", "rev-parse", "HEAD"], cwd=ROOT, check=True,
            capture_output=True, text=True,
        ).stdout.strip()
        (evidence_dir / "source-revision.txt").write_text(revision + "\n", encoding="utf-8")
        (evidence_dir / "make-ci.status.txt").write_text("0\n", encoding="utf-8")
        (evidence_dir / "make-ci.stdout").write_text(healthy_ci_output(), encoding="utf-8")
        (evidence_dir / "pgm01-schema.status.txt").write_text("125\n", encoding="utf-8")
        (evidence_dir / "pgm01-schema.stdout").write_text(
            "ordinary-output\n", encoding="utf-8"
        )
        (evidence_dir / "pgm01-validator.status.txt").write_text("3\n", encoding="utf-8")
        outcomes = {item["name"]: item for item in MODULE.command_outcomes(evidence_dir)}
        assert outcomes["make-ci"] == {
            "name": "make-ci",
            "status": "passed",
            "exitCode": 0,
        }
        assert outcomes["pgm01-schema"] == {
            "name": "pgm01-schema",
            "status": "skipped-unavailable",
            "exitCode": 125,
        }
        assert outcomes["pgm01-validator"] == {
            "name": "pgm01-validator",
            "status": "failed",
            "exitCode": 3,
        }
        assert outcomes["make-spec"] == {
            "name": "make-spec",
            "status": "inconclusive",
            "exitCode": None,
        }
        assert MODULE.classify_result("final", [outcomes["make-ci"]])[0] == "inconclusive"
        assert MODULE.classify_result("provisional", [outcomes["make-ci"]])[0] == "inconclusive"
        assert MODULE.classify_result("sealed-failed", [outcomes["make-ci"]])[0] == "error"
        assert MODULE.classify_result("final", [outcomes["pgm01-schema"]])[0] == "inconclusive"
        assert MODULE.classify_result("final", [outcomes["pgm01-validator"]])[0] == "error"

        (evidence_dir / "evidence-envelope.json").write_text("{}\n", encoding="utf-8")
        for name in FINALIZER.CHECKS:
            (evidence_dir / f"{name}.status.txt").write_text("0\n", encoding="utf-8")
            output = (
                '{"errors": [], "valid": true}\n'
                if name in {
                    "input-schema", "manifest-schema", "pgm01-schema", "pgm01-validator",
                    "sealed-pgm01-schema", "sealed-pgm01-validator",
                }
                else "verified\n"
            )
            (evidence_dir / f"{name}.stdout").write_text(output, encoding="utf-8")
            (evidence_dir / f"{name}.stderr").write_text("", encoding="utf-8")
        (evidence_dir / "make-ci.stdout").write_text(healthy_ci_output(), encoding="utf-8")
        (evidence_dir / "msrv.stdout").write_text(healthy_msrv_output(), encoding="utf-8")
        (evidence_dir / "rustdoc.stderr").write_text(
            f"Generated /tmp/doc/tl_rewrite/index.html\n"
            f"observed rustdoc index SHA-256 {'a' * 64}\n",
            encoding="utf-8",
        )
        traces = trace_total()
        (evidence_dir / "quire-coverage.stdout").write_text(
            f"Coverage: {traces}/{traces} rows backed (100%)\n", encoding="utf-8"
        )
        (evidence_dir / "make-spec.stdout").write_text(
            f"strict traceability coverage is complete: {traces}/{traces}\n", encoding="utf-8"
        )
        (evidence_dir / "default-dependencies.stdout").write_text(
            "tl-rewrite v0.1.0\n"
            + "\n".join(
                revision[:8]
                for revision in re.findall(
                    r'rev\s*=\s*"([0-9a-f]{40})"',
                    (ROOT / "Cargo.toml").read_text(encoding="utf-8"),
                )
            )
            + "\n",
            encoding="utf-8",
        )
        (evidence_dir / "corpus-integrity.stdout").write_text(
            "WEST corpus checksum census passed\n", encoding="utf-8"
        )
        (evidence_dir / "diff-integrity.stdout").write_text(
            f"diff-integrity gate passed for origin/main...{revision}\n", encoding="utf-8"
        )
        retained = FINALIZER.summary(evidence_dir)
        assert retained["overallStatus"] == "passed"
        assert FINALIZER.positive_output(evidence_dir, "make-ci") == FINALIZER.CORROBORATED
        assert FINALIZER.validate_tool_identity(evidence_dir) == []
        profile_value = json.loads(
            (evidence_dir / "collection-input.json").read_text(encoding="utf-8")
        )
        profile_value["tools"]["identities"]["git"]["sha256"] = "0" * 64
        (evidence_dir / "collection-input.json").write_text(
            json.dumps(profile_value), encoding="utf-8"
        )
        assert FINALIZER.validate_tool_identity(evidence_dir), (
            "retained tool-identity validation accepted a changed identity"
        )
        profile_value["tools"]["identities"] = locked_identities["tools"]
        (evidence_dir / "collection-input.json").write_text(
            json.dumps(profile_value), encoding="utf-8"
        )

        healthy_ci = healthy_ci_output()
        gate_signatures = (
            "qualified-tool-identities gate passed", "fmt-check gate passed",
            "lint gate passed", "Rust test gate passed", "corpus-integrity gate passed",
            "deny gate passed", "audit-unsafe gate passed", "evidence-tool gate passed",
            "spec gate passed", "msrv gate passed", "rustdoc gate passed",
        )
        for signature in gate_signatures:
            assert_output_rejected(
                evidence_dir, "make-ci", healthy_ci.replace(signature + "\n", "", 1)
            )
        for signature in (
            "licenses ok",
            "sources ok",
            f"advisories ok at {'b' * 40}",
        ):
            assert_output_rejected(
                evidence_dir, "make-ci", healthy_ci.replace(signature + "\n", "", 1)
            )
        target_count = len(PROPAGATION.PROBE_TARGETS)
        policies = policy_test_count()
        for expected, fabricated_value in (
            (
                f"all {target_count} mandatory local-CI targets propagate failures",
                f"all {target_count - 1} mandatory local-CI targets propagate failures",
            ),
            (
                f"all {policies} evidence-policy behavior tests passed",
                f"all {policies - 1} evidence-policy behavior tests passed",
            ),
            (
                f"strict traceability coverage is complete: {traces}/{traces}",
                f"strict traceability coverage is complete: {traces - 1}/{traces - 1}",
            ),
        ):
            assert_output_rejected(
                evidence_dir, "make-ci", healthy_ci.replace(expected, fabricated_value, 1)
            )
        assert_output_rejected(
            evidence_dir, "make-ci", healthy_ci.replace("Generated ", "Documented ", 1)
        )
        assert_output_rejected(
            evidence_dir,
            "make-ci",
            healthy_ci.replace("/doc/tl_rewrite/index.html", "/doc/other/index.html", 1),
        )
        rustdoc_stderr = evidence_dir / "rustdoc.stderr"
        rustdoc_output = rustdoc_stderr.read_text(encoding="utf-8")
        rustdoc_stderr.write_text(
            "Generated /tmp/doc/tl_rewrite/index.html\n", encoding="utf-8"
        )
        assert FINALIZER.positive_output(evidence_dir, "rustdoc") == FINALIZER.REJECTED
        rustdoc_stderr.write_text(rustdoc_output, encoding="utf-8")
        assert_output_rejected(evidence_dir, "corpus-integrity", "verified\n")
        assert_output_rejected(evidence_dir, "diff-integrity", "verified\n")
        dependency_output = (evidence_dir / "default-dependencies.stdout").read_text(
            encoding="utf-8"
        )
        dependency_revisions = re.findall(
            r'rev\s*=\s*"([0-9a-f]{40})"', (ROOT / "Cargo.toml").read_text(encoding="utf-8")
        )
        assert len(dependency_revisions) >= 2
        for dependency_revision in dependency_revisions[:2]:
            assert_output_rejected(
                evidence_dir,
                "default-dependencies",
                dependency_output.replace(dependency_revision[:8], "deadbeef", 1),
            )
        schema_output = (evidence_dir / "input-schema.stdout").read_text(encoding="utf-8")
        assert_output_rejected(
            evidence_dir, "input-schema", schema_output.replace("true", "false", 1)
        )
        fabricated = "test result: ok. 1 passed; 0 failed; 0 ignored\n" * 16
        (evidence_dir / "make-ci.stdout").write_text(fabricated, encoding="utf-8")
        assert FINALIZER.positive_output(evidence_dir, "make-ci") == FINALIZER.REJECTED
        (evidence_dir / "make-ci.stdout").write_text(healthy_ci_output(), encoding="utf-8")
        (evidence_dir / "make-ci.stdout").write_text("", encoding="utf-8")
        assert FINALIZER.summary(evidence_dir)["overallStatus"] == "failed"
        (evidence_dir / "make-ci.stdout").write_text(healthy_ci_output(), encoding="utf-8")
        (evidence_dir / "rustdoc.stderr").write_text("", encoding="utf-8")
        assert FINALIZER.summary(evidence_dir)["overallStatus"] == "failed"
        (evidence_dir / "rustdoc.stderr").write_text(
            f"Generated /tmp/doc/tl_rewrite/index.html\n"
            f"observed rustdoc index SHA-256 {'a' * 64}\n",
            encoding="utf-8",
        )
        profile_input = evidence_dir / "collection-input.json"
        profile_input.write_text("{}\n", encoding="utf-8")
        try:
            FINALIZER.summary(evidence_dir)
        except ValueError:
            pass
        else:
            raise AssertionError("v2-era record without qualificationProfile passed")
        profile_input.write_text(
            json.dumps({
                "qualificationProfile": "tl-rewrite.evidence-qualification/v2",
                "tools": {
                    "identities": locked_identities["tools"],
                    "runtimeIdentities": locked_identities["runtimeIdentities"],
                },
            }),
            encoding="utf-8",
        )
        assert retained["finalEnvelopeValidated"] is True
        (evidence_dir / "pgm01-validator.stderr").write_text(
            "governance validation error\n", encoding="utf-8"
        )
        contradicted = FINALIZER.summary(evidence_dir)
        assert contradicted["overallStatus"] == "failed"
        (evidence_dir / "pgm01-validator.stderr").unlink()
        (evidence_dir / "rustdoc.status.txt").write_text("1\n", encoding="utf-8")
        rederived = FINALIZER.summary(evidence_dir)
        assert rederived["overallStatus"] == "failed"
        assert rederived != retained
        (evidence_dir / "newgate.status.txt").write_text("1\n", encoding="utf-8")
        censused = FINALIZER.summary(evidence_dir)
        assert any(item["name"] == "newgate" for item in censused["outcomes"])
        assert censused["overallStatus"] == "failed"

        (evidence_dir / "evidence-envelope.json").write_text(
            json.dumps({"result": {"status": "conclusive"}}) + "\n",
            encoding="utf-8",
        )
        contradictory_summary = FINALIZER.summary(evidence_dir)
        (evidence_dir / "collection-summary.json").write_text(
            json.dumps(contradictory_summary, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        contradictory = subprocess.run(
            [
                sys.executable,
                str(ROOT / "scripts" / "finalize_collection.py"),
                "--check",
                str(evidence_dir),
            ],
            check=False,
            capture_output=True,
        )
        assert contradictory.returncode != 0

        missing_summary = subprocess.run(
            [
                sys.executable,
                str(ROOT / "scripts" / "finalize_collection.py"),
                "--check",
                str(evidence_dir),
            ],
            check=False,
            capture_output=True,
        )
        assert missing_summary.returncode != 0

        missing_check_directory = subprocess.run(
            [
                sys.executable,
                str(ROOT / "scripts" / "finalize_collection.py"),
                "--check",
            ],
            check=False,
            capture_output=True,
        )
        assert missing_check_directory.returncode == 2
        assert b"usage:" in missing_check_directory.stderr

        artifact = evidence_dir / "make-ci.stdout"
        artifact.write_text("passed\n", encoding="utf-8")
        manifest = {
            "artifacts": [
                {
                    "path": artifact.name,
                    "sha256": VERIFIER.sha256(artifact),
                    "size": artifact.stat().st_size,
                }
            ]
        }
        (evidence_dir / "evidence-manifest.json").write_text(
            json.dumps(manifest), encoding="utf-8"
        )
        checksum = evidence_dir.with_suffix(".sha256")
        checksum_contents = "".join(
            f"{VERIFIER.sha256(path)}  {evidence_dir.name}/{path.name}\n"
            for path in sorted(evidence_dir.iterdir())
            if path.is_file()
        )
        checksum.write_text(checksum_contents, encoding="utf-8")
        assert VERIFIER.verify(verification_dir) == []
        checksum.write_text(
            checksum_contents.replace(
                f"  {evidence_dir.name}/", f"  unrelated/{evidence_dir.name}/", 1
            ),
            encoding="utf-8",
        )
        assert any(
            "escapes evidence directory" in error
            for error in VERIFIER.verify(verification_dir)
        ), "manifest verifier accepted a checksum path outside the record prefix"
        checksum.write_text(checksum_contents, encoding="utf-8")
        added = evidence_dir / "reviewer-attestation.txt"
        added.write_text("FABRICATED\n", encoding="utf-8")
        assert any("unlisted" in error for error in VERIFIER.verify(verification_dir))
        added.unlink()
        symlink = evidence_dir / "symlink.txt"
        symlink.symlink_to(artifact)
        assert any("symlink" in error for error in VERIFIER.verify(verification_dir))
        symlink.unlink()
        artifact.write_text("FABRICATED\n", encoding="utf-8")
        assert VERIFIER.verify(verification_dir)
        rejected = subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "verify_evidence_manifest.py"),
             str(verification_dir)], check=False, capture_output=True,
        )
        assert rejected.returncode != 0, "manifest verifier main accepted corrupt evidence"
    with tempfile.TemporaryDirectory() as directory:
        historical = Path(directory) / assured.name
        historical.mkdir()
        shutil.copy2(assured / "collection-input.json", historical / "collection-input.json")
        shutil.copy2(assured / "source-revision.txt", historical / "source-revision.txt")
        (historical / "evidence-envelope.json").write_text("{}\n", encoding="utf-8")
        (historical / "diff-integrity.status.txt").write_text("0\n", encoding="utf-8")
        (historical / "diff-integrity.stdout").write_text("verified\n", encoding="utf-8")
        assert FINALIZER.positive_output(
            historical, "diff-integrity"
        ) == FINALIZER.NOT_APPLICABLE
        historical_summary = FINALIZER.summary(historical)
        diff_outcome = next(
            item for item in historical_summary["outcomes"] if item["name"] == "diff-integrity"
        )
        assert diff_outcome["status"] == "inconclusive"
        assert historical_summary["overallStatus"] == "inconclusive"
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        failing = root / "test_fails.py"
        failing.write_text("raise SystemExit(7)\n", encoding="utf-8")
        runner = subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "run_policy_tests.py"),
             "--directory", str(root)], check=False, capture_output=True,
        )
        assert runner.returncode != 0, "policy runner swallowed a failing test"
        corpus = root / "corpus"
        corpus.mkdir()
        (corpus / "SHA256SUMS").write_text("0" * 64 + "  missing.txt\n", encoding="utf-8")
        checker = subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "check_corpus.py"),
             "--directory", str(corpus)], check=False, capture_output=True,
        )
        assert checker.returncode != 0, "corpus checker exit contract accepted missing data"
        builder = subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "build_evidence_envelope.py"),
             str(root / "missing"), "final"],
            check=False, capture_output=True,
        )
        assert builder.returncode != 0, "evidence builder main accepted missing artifacts"
    with tempfile.TemporaryDirectory() as directory:
        clone = Path(directory) / "repository"
        subprocess.run(
            ["/usr/bin/git", "clone", "--quiet", "--no-hardlinks", str(ROOT), str(clone)],
            check=True,
        )
        for script in (ROOT / "scripts").iterdir():
            if script.suffix in {".py", ".sh"}:
                shutil.copy2(script, clone / "scripts" / script.name)
        shutil.copy2(ROOT / "evidence" / "RETRACTIONS.json", clone / "evidence" / "RETRACTIONS.json")
        verifier = clone / "scripts" / "verify_evidence.sh"
        verifier_text = verifier.read_text(encoding="utf-8")
        for unrelated in (
            "/usr/bin/python3 scripts/check_assurance_anchor.py",
            "/usr/bin/python3 scripts/check_evidence_root.py",
            "/usr/bin/python3 scripts/verify_evidence_history.py",
            "/usr/bin/python3 scripts/finalize_collection.py --check \"$evidence_dir\"",
        ):
            verifier_text = verifier_text.replace(unrelated, "/usr/bin/true")
        verifier.write_text(verifier_text, encoding="utf-8")
        with (clone / ".gitignore").open("a", encoding="utf-8") as ignore:
            ignore.write("\nevidence/**/PLANTED-EXIT-CONTRACT-*\n")
        subprocess.run(["/usr/bin/git", "add", "scripts", "evidence/RETRACTIONS.json", ".gitignore"], cwd=clone, check=True)
        subprocess.run(
            ["/usr/bin/git", "-c", "user.name=Policy Test", "-c",
             "user.email=policy@example.invalid", "commit", "-qm", "test current verifier"],
            cwd=clone, check=True,
        )
        record = next(path for path in (clone / "evidence").glob("tl-rewrite-v01-*") if path.is_dir())
        planted = record / f"PLANTED-EXIT-CONTRACT-{os.getpid()}.txt"
        planted.write_text("FABRICATED\n", encoding="utf-8")
        shell = subprocess.run(
            ["/usr/bin/bash", "scripts/verify_evidence.sh"], cwd=clone, check=False,
            capture_output=True, text=True,
        )
        assert shell.returncode != 0, "evidence shell verifier exit contract was gutted"
        assert "unlisted retained artifact" in shell.stderr, (
            f"clean-tree preflight stopped the census-loop behavior test: {shell.stderr}"
        )
    print("evidence outcome behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
