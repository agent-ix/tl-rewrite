#!/usr/bin/env python3
"""Build tl-rewrite's PGM-01 collection input, manifest, and envelope."""

from __future__ import annotations

import datetime as dt
import hashlib
import json
import platform
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
PGM01_POLICY_REVISION = "7dac9d8c19952412b56a0347387666e2ca81e01d"
PGM01_SCHEMA_DIGEST = "0946e235e9e4b0fa79e9b9ec27ae157b303c17de0a9408d3cc04968fb7152256"
TL_SYNTAX_REVISION = "5e59a26d71b4b5d79623850cda50010e18a90dad"
TL_MLTL_REVISION = "a9b7847199c1d846abd7b67901cd6836374ccee2"
WEST_REVISION = "21cd99ab2e6095a099dd179029cfdeb54268ad3f"
INPUT_SCHEMA = ROOT / "schemas" / "tl-rewrite-evidence-input-v1.schema.json"
MANIFEST_SCHEMA = ROOT / "schemas" / "tl-rewrite-evidence-manifest-v1.schema.json"
COLLECTOR = ROOT / "scripts" / "collect_evidence.sh"
BUILDER = Path(__file__).resolve()
VALIDATOR = ROOT / "scripts" / "validate_json_schema.py"
COMMANDS = (
    "make-ci",
    "make-spec",
    "quire-coverage",
    "rustdoc",
    "default-dependencies",
    "corpus-integrity",
    "diff-integrity",
)


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest(value: str) -> dict[str, str]:
    return {"algorithm": "sha256", "value": value}


def schema_identity(name: str, path: Path) -> dict[str, object]:
    return {"id": name, "version": "v1", "digest": digest(sha256_file(path))}


def write_json(path: Path, value: Any) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def first_line(path: Path) -> str:
    return path.read_text(encoding="utf-8").splitlines()[0]


def outcomes(directory: Path) -> list[dict[str, object]]:
    values = []
    for name in COMMANDS:
        code = int((directory / f"{name}.status.txt").read_text().strip())
        values.append({"name": name, "status": "passed" if code == 0 else "failed", "exitCode": code})
    return values


def parameters_digest() -> str:
    paths = (
        ROOT / "Cargo.toml",
        ROOT / "Cargo.lock",
        ROOT / "Makefile",
        ROOT / "rust-toolchain.toml",
        ROOT / "src" / "catalog.rs",
        ROOT / "corpus" / "west-v1" / "SHA256SUMS",
        ROOT / "corpus" / "west-v1" / "manifest.json",
        COLLECTOR,
        BUILDER,
        VALIDATOR,
        INPUT_SCHEMA,
        MANIFEST_SCHEMA,
    )
    state = hashlib.sha256()
    for path in paths:
        state.update(str(path.relative_to(ROOT)).encode())
        state.update(b"\0")
        state.update(path.read_bytes())
        state.update(b"\0")
    return state.hexdigest()


def build(directory: Path) -> None:
    directory = directory.resolve()
    relative = str(directory.relative_to(ROOT))
    revision = (directory / "source-revision.txt").read_text().strip()
    metadata = json.loads((directory / "metadata.json").read_text())
    package = next(item for item in metadata["packages"] if item["name"] == "tl-rewrite")
    recorded_at = dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    quire_version = json.loads((directory / "quire-provenance.json").read_text())["cli"]["version"]

    collection_input = {
        "schemaVersion": "tl-rewrite.evidence-input/v1",
        "sourceRevision": revision,
        "sourceState": "clean",
        "commands": [
            "make ci",
            "make spec",
            "quire coverage --scope . --strict",
            "RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features",
            "cargo tree --no-default-features --edges normal",
            "make check-corpus",
            f"git diff --check origin/main...{revision}",
            "validate local evidence schemas and exact merged PGM-01 envelope",
        ],
        "tools": {
            "cargo": first_line(directory / "cargo-version.txt"),
            "jsonschema": (directory / "jsonschema-version.txt").read_text().strip(),
            "python": (directory / "python-version.txt").read_text().strip(),
            "quire": quire_version,
            "rustc": first_line(directory / "rustc-version.txt"),
        },
        "pgm01": {
            "policy": "ix://agent-ix/quire-contract-ir/PGM-01",
            "candidateRevision": PGM01_POLICY_REVISION,
            "envelopeSchema": "quire.derivation-evidence/v1",
            "envelopeSchemaDigest": digest(PGM01_SCHEMA_DIGEST),
        },
        "dependencies": {
            "tlSyntaxRevision": TL_SYNTAX_REVISION,
            "tlMltlRevision": TL_MLTL_REVISION,
            "cargoLockDigest": digest(sha256_file(ROOT / "Cargo.lock")),
        },
        "catalog": {
            "version": "tl-rewrite-rules/v1",
            "sourceDigest": digest(sha256_file(ROOT / "src" / "catalog.rs")),
        },
        "corpus": {
            "revision": "tl-rewrite-corpus/v1",
            "westSourceRevision": WEST_REVISION,
            "manifestDigest": digest(sha256_file(ROOT / "corpus" / "west-v1" / "manifest.json")),
            "checksumDigest": digest(sha256_file(ROOT / "corpus" / "west-v1" / "SHA256SUMS")),
            "licenseDigest": digest(sha256_file(ROOT / "corpus" / "west-v1" / "LICENSE")),
        },
    }
    input_path = directory / "collection-input.json"
    write_json(input_path, collection_input)

    excluded = {
        "collection-input.json", "evidence-envelope.json", "evidence-manifest.json",
        "input-schema.stdout", "input-schema.stderr", "input-schema.status.txt",
        "manifest-schema.stdout", "manifest-schema.stderr", "manifest-schema.status.txt",
        "pgm01-schema.stdout", "pgm01-schema.stderr", "pgm01-schema.status.txt",
        "pgm01-validator.stdout", "pgm01-validator.stderr", "pgm01-validator.status.txt",
    }
    artifacts = [
        {"path": path.name, "sha256": sha256_file(path), "size": path.stat().st_size}
        for path in sorted(directory.iterdir(), key=lambda item: item.name)
        if path.is_file() and path.name not in excluded
    ]
    command_outcomes = outcomes(directory)
    passed = all(item["status"] == "passed" for item in command_outcomes)
    limitations = [
        "manual-dispatch remote CI is not part of this local envelope",
        "38 enabled closed-trace rules have exhaustive small-domain evidence; two WEST Theorem 3 rules remain excluded",
        "ten selected WEST inputs are exercised without claiming universal proof or WEST qualification",
        "online-prefix rewriting, independent human approval, and the source-release decision remain pending",
    ]
    if not passed:
        limitations.append("one or more locally collected commands failed")
    manifest = {
        "schemaVersion": "tl-rewrite.evidence-manifest/v1",
        "sourceRevision": revision,
        "collectedAt": recorded_at,
        "outcomes": command_outcomes,
        "artifacts": artifacts,
        "limitations": limitations,
    }
    manifest_path = directory / "evidence-manifest.json"
    write_json(manifest_path, manifest)

    host = next(
        line.split(": ", 1)[1]
        for line in (directory / "rustc-version.txt").read_text().splitlines()
        if line.startswith("host: ")
    )
    envelope = {
        "schemaVersion": "quire.derivation-evidence/v1",
        "recordId": directory.name,
        "recordedAt": recorded_at,
        "producer": {
            "name": "tl-rewrite-evidence-collector",
            "version": package["version"],
            "sourceRevision": revision,
            "executableDigest": digest(sha256_file(COLLECTOR)),
            "invocation": ["bash", "scripts/collect_evidence.sh", relative],
        },
        "inputs": [{
            "role": "evidence-collection-input", "uri": "collection-input.json",
            "mediaType": "application/json",
            "schema": schema_identity("tl-rewrite.evidence-input", INPUT_SCHEMA),
            "contentDigest": digest(sha256_file(input_path)),
        }],
        "backend": {"kind": "none", "reason": "deterministic packaging; invoked tools are identified in the input"},
        "outputs": [{
            "role": "tl-rewrite-evidence-manifest", "uri": "evidence-manifest.json",
            "mediaType": "application/json",
            "schema": schema_identity("tl-rewrite.evidence-manifest", MANIFEST_SCHEMA),
            "contentDigest": digest(sha256_file(manifest_path)),
        }],
        "parametersDigest": digest(parameters_digest()),
        "environment": {
            "targetTriple": host,
            "operatingSystem": platform.platform(),
            "toolchain": collection_input["tools"]["rustc"],
            "dependenciesDigest": digest(sha256_file(ROOT / "Cargo.lock")),
        },
        "provenance": {
            "repository": "https://github.com/agent-ix/tl-rewrite",
            "sourceRevision": revision,
            "candidateRevision": revision,
            "contributionMethod": "agent-assisted",
            "reviewers": ["@kreneskyp"],
        },
        "result": {
            "status": "conclusive" if passed else "error",
            "summary": "all local tl-rewrite checks passed; human gates remain pending" if passed else "one or more local tl-rewrite checks failed",
            "requirementRefs": ["PGM-01-R08", "PGM-01-R09", "MP-001"],
        },
        "extensions": {"dev.agent-ix.tl-rewrite": {
            "componentClass": "analysis-evidence-tool",
            "catalogVersion": "tl-rewrite-rules/v1",
            "corpusRevision": "tl-rewrite-corpus/v1",
            "envelopeSchemaDigest": PGM01_SCHEMA_DIGEST,
            "pgm01CandidateRevision": PGM01_POLICY_REVISION,
            "reviewState": "pending",
            "sourceState": "clean",
        }},
    }
    write_json(directory / "evidence-envelope.json", envelope)


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: build_evidence_envelope.py EVIDENCE_DIR", file=sys.stderr)
        return 2
    build(Path(sys.argv[1]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
