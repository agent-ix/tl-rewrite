#!/usr/bin/env python3
"""Behavior tests for evidence outcome classification."""

from __future__ import annotations

import importlib.util
import json
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


def main() -> int:
    with tempfile.TemporaryDirectory() as directory:
        evidence_dir = Path(directory)
        (evidence_dir / "make-ci.status.txt").write_text("0\n", encoding="utf-8")
        (evidence_dir / "make-ci.stdout").write_text("passed\n", encoding="utf-8")
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
        retained = FINALIZER.summary(evidence_dir)
        assert retained["overallStatus"] == "passed"
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
                f"{VERIFIER.sha256(path)}  {path}\n"
                for path in sorted(evidence_dir.iterdir())
                if path.is_file()
            ),
            encoding="utf-8",
        )
        assert VERIFIER.verify(evidence_dir) == []
        added = evidence_dir / "reviewer-attestation.txt"
        added.write_text("FABRICATED\n", encoding="utf-8")
        assert any("unlisted" in error for error in VERIFIER.verify(evidence_dir))
        added.unlink()
        artifact.write_text("FABRICATED\n", encoding="utf-8")
        assert VERIFIER.verify(evidence_dir)
    print("evidence outcome behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
