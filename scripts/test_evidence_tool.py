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
    return healthy_test_output(2) + (
        "all 11 mandatory local-CI targets propagate failures\n"
        "all 8 evidence-policy behavior tests passed\n"
        "strict traceability coverage is complete: 52/52\n"
        "licenses ok\nsources ok\n"
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
        f"rustdoc index SHA-256 {'a' * 64}\n"
        "verify-evidence gate passed\n"
    )


def healthy_msrv_output() -> str:
    return healthy_test_output(1)


def main() -> int:
    if sys.flags.optimize or os.environ.get("PYTHONOPTIMIZE"):
        return 2
    assured = ROOT / "evidence" / "tl-rewrite-v01-1b08a6c9e7bc-20260831T203039Z"
    assert FINALIZER.validate_content_digests(assured) == []
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
    with tempfile.TemporaryDirectory(dir=ROOT) as directory:
        evidence_dir = Path(directory) / "record"
        evidence_dir.mkdir()
        verification_dir = evidence_dir.relative_to(ROOT)
        (evidence_dir / "collection-input.json").write_text(
            json.dumps({"qualificationProfile": "tl-rewrite.evidence-qualification/v2"}),
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
            f"rustdoc index SHA-256 {'a' * 64}\n",
            encoding="utf-8",
        )
        (evidence_dir / "quire-coverage.stdout").write_text(
            "Coverage: 52/52 rows backed (100%)\n", encoding="utf-8"
        )
        (evidence_dir / "make-spec.stdout").write_text(
            "strict traceability coverage is complete: 52/52\n", encoding="utf-8"
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
        assert FINALIZER.positive_output(evidence_dir, "make-ci")
        fabricated = "test result: ok. 1 passed; 0 failed; 0 ignored\n" * 16
        (evidence_dir / "make-ci.stdout").write_text(fabricated, encoding="utf-8")
        assert not FINALIZER.positive_output(evidence_dir, "make-ci")
        (evidence_dir / "make-ci.stdout").write_text(healthy_ci_output(), encoding="utf-8")
        (evidence_dir / "make-ci.stdout").write_text("", encoding="utf-8")
        assert FINALIZER.summary(evidence_dir)["overallStatus"] == "failed"
        (evidence_dir / "make-ci.stdout").write_text(healthy_ci_output(), encoding="utf-8")
        (evidence_dir / "rustdoc.stderr").write_text("", encoding="utf-8")
        assert FINALIZER.summary(evidence_dir)["overallStatus"] == "failed"
        (evidence_dir / "rustdoc.stderr").write_text(
            f"Generated /tmp/doc/tl_rewrite/index.html\n"
            f"rustdoc index SHA-256 {'a' * 64}\n",
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
            json.dumps({"qualificationProfile": "tl-rewrite.evidence-qualification/v2"}),
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
        checksum.write_text(
            "".join(
                f"{VERIFIER.sha256(path)}  {verification_dir / path.name}\n"
                for path in sorted(evidence_dir.iterdir())
                if path.is_file()
            ),
            encoding="utf-8",
        )
        assert VERIFIER.verify(verification_dir) == []
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
