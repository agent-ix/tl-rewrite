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
        original + "\nexport MAKEFLAGS = -i\n",
        original + "\nexport MAKEFLAGS := -i\n",
        original + "\noverride MAKEFLAGS = -i\n",
        original + "\noverride MAKEFLAGS += -i\n",
        original + "\nMAKEFLAGS ::= -i\n",
        original + "\nMAKEFLAGS :::= -i\n",
        original + "\nMAKEFLAGS != printf -- -i\n",
        original + "\noverride MAKEFLAGS ::= -i\n",
        original + "\nexport MAKEFLAGS!=printf -- -i\n",
        original + "\n%: MAKEFLAGS ::= -i\n",
        original + "\noverride define MAKEFLAGS\n-i\nendef\n",
        original + "\n.ONESHELL:\n",
        original + "\n.DEFAULT:\n\ttrue\n",
        original + "\nSHELL = /usr/bin/true\n",
        original + "\n.SHELLFLAGS := -c\n",
        original + "\ntest: SHELL = /usr/bin/true\n",
        original + "\ntest: private .SHELLFLAGS ::= -c\n",
        original + "\n$(eval MAKEFLAGS := -i)\n",
        original + "\n$(eval .DEFAULT:)\n",
        original + "\n${eval MAKEFLAGS := -i}\n",
        original + "\n${eval .DEFAULT:}\n",
        original + "\ninclude imported.mk\n",
        original + "\n-include optional.mk\n",
        original + "\nsinclude optional.mk\n",
        original + "\n$(eval include imported.mk)\n",
        original + "\n${eval include imported.mk}\n",
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
    with tempfile.TemporaryDirectory() as directory:
        swallowed = Path(directory) / "Makefile"
        swallowed.write_text(".PHONY: fmt-check\nfmt-check:\n\t-false\n", encoding="utf-8")
        assert MODULE.probe_command_positions(swallowed), (
            "command-position probe accepted an ignored failure"
        )
        oneshell = Path(directory) / "Makefile.oneshell"
        oneshell.write_text(
            ".ONESHELL:\n.PHONY: fmt-check\nfmt-check:\n\tfalse\n\ttrue\n",
            encoding="utf-8",
        )
        assert MODULE.probe_command_positions(oneshell), (
            "command-position probe did not inherit global .ONESHELL behavior"
        )
        default = Path(directory) / "Makefile.default"
        default.write_text(".DEFAULT:\n\ttrue\n", encoding="utf-8")
        default_result = subprocess.run(
            ["/usr/bin/make", "--no-print-directory", "-f", str(default), "fabricated"],
            check=False, capture_output=True,
        )
        assert default_result.returncode == 0, ".DEFAULT fixture did not exercise fallback success"
        assert MODULE.inspect(default), ".DEFAULT execution-control mutation escaped inspection"
        brace_eval = Path(directory) / "Makefile.brace-eval"
        brace_eval.write_text(
            ".PHONY: fabricated\nfabricated:\n\tfalse\n${eval .IGNORE:}\n",
            encoding="utf-8",
        )
        brace_result = subprocess.run(
            ["/usr/bin/make", "--no-print-directory", "-f", str(brace_eval), "fabricated"],
            check=False, capture_output=True,
        )
        assert brace_result.returncode == 0, "brace-form eval fixture did not ignore failure"
        assert MODULE.inspect(brace_eval), "brace-form GNU Make eval escaped inspection"
    ignored_make = subprocess.run(
        ["make", "--no-print-directory", "-i", "ci"], cwd=ROOT, check=False,
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        env={key: value for key, value in os.environ.items() if key != "MAKEFLAGS"},
    )
    assert ignored_make.returncode != 0
    with tempfile.TemporaryDirectory() as directory:
        disposable = Path(directory)
        (disposable / "src").mkdir()
        (disposable / "scripts").mkdir()
        (disposable / "src" / "unsafe_policy_probe.rs").write_text(
            "fn probe() { unsafe { core::hint::unreachable_unchecked() } }\n",
            encoding="utf-8",
        )
        (disposable / "scripts" / "unsafe_comment_baseline.txt").write_text(
            "", encoding="utf-8"
        )
        unsafe_result = subprocess.run(
            ["/usr/bin/bash", str(ROOT / "scripts" / "check_unsafe_comments.sh")],
            cwd=disposable,
            check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )
        assert unsafe_result.returncode != 0, "unsafe-comment shell gate was neutered"
    print("failure-propagation policy behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
