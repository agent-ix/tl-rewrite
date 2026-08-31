#!/usr/bin/env python3
"""Mutation tests for the local-CI failure-propagation policy."""

from __future__ import annotations

import importlib.util
import os
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location(
    "check_failure_propagation", ROOT / "scripts" / "check_failure_propagation.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def main() -> int:
    original = (ROOT / "Makefile").read_text(encoding="utf-8")
    assert MODULE.inspect(ROOT / "Makefile") == []
    mutations = [
        original.replace("\tcargo clippy", "\t -cargo clippy", 1),
        original.replace("\tcargo test", "\tcargo test || true", 1),
        original.replace("\tcargo test", "\tcargo test; true", 1),
        original + "\n.IGNORE: test\n",
        original + "\nMAKEFLAGS += -i\n",
        original.replace("ci: ", "ci: fabricated ", 1),
    ]
    with tempfile.TemporaryDirectory() as directory:
        for index, mutated in enumerate(mutations):
            path = Path(directory) / f"Makefile.{index}"
            path.write_text(mutated, encoding="utf-8")
            assert MODULE.inspect(path), f"mutation {index} escaped inspection"
            actual = subprocess.run(
                [sys.executable, str(ROOT / "scripts" / "check_failure_propagation.py"),
                 "--makefile", str(path), "--static-only"],
                check=False, capture_output=True,
            )
            assert actual.returncode != 0, f"checker exit contract accepted mutation {index}"
        ignored = Path(directory) / "tests" / "ignored.rs"
        ignored.parent.mkdir()
        ignored.write_text("#[test]\n#[cfg_attr(test, ignore)]\nfn disabled() {}\n", encoding="utf-8")
        assert MODULE.inspect(ROOT / "Makefile", Path(directory))
    assert MODULE.probe_command_positions(ROOT / "Makefile") == []
    for value in ("i", "ik", "-i", "--ignore-errors"):
        assert MODULE.makeflags_ignore_errors(value)
    ignored_make = subprocess.run(
        ["make", "--no-print-directory", "-i", "ci"], cwd=ROOT, check=False,
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        env={key: value for key, value in os.environ.items() if key != "MAKEFLAGS"},
    )
    assert ignored_make.returncode != 0
    with tempfile.TemporaryDirectory() as directory:
        shim = Path(directory) / "cargo"
        shim.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
        shim.chmod(0o755)
        shadowed_env = dict(os.environ)
        shadowed_env.pop("MAKEFLAGS", None)
        shadowed_env["PATH"] = f"{directory}:{shadowed_env['PATH']}"
        shadowed = subprocess.run(
            ["/usr/bin/make", "--no-print-directory", "ci"], cwd=ROOT,
            check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
            env=shadowed_env,
        )
        assert shadowed.returncode != 0, "PATH-shadowed cargo bypassed local CI"
    print("failure-propagation policy behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
