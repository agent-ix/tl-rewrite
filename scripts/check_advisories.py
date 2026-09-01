#!/usr/bin/env python3
"""Run cargo-deny and identify the exact advisory database revision it used."""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import Mapping


REPOSITORY = "https://github.com/RustSec/advisory-db"
ROOT = Path(__file__).resolve().parent.parent
DENY_CONFIG = ROOT / "deny.toml"


def resolved_cargo_home(environment: Mapping[str, str] = os.environ) -> Path:
    cargo_home = environment.get("CARGO_HOME")
    if cargo_home:
        path = Path(cargo_home)
        if not path.is_absolute():
            raise OSError("CARGO_HOME is not absolute")
        return path
    home = environment.get("HOME")
    if not home:
        raise OSError("neither CARGO_HOME nor HOME identifies the advisory database")
    path = Path(home)
    if not path.is_absolute():
        raise OSError("HOME is not absolute")
    return path / ".cargo"


def database_root(
    config: Path = DENY_CONFIG, environment: Mapping[str, str] = os.environ
) -> Path:
    matches = re.findall(
        r'^db-path\s*=\s*"([^"]+)"\s*$',
        config.read_text(encoding="utf-8"),
        re.MULTILINE,
    )
    if len(matches) != 1 or not matches[0].startswith("$CARGO_HOME/"):
        raise ValueError("deny.toml advisory db-path is not rooted at $CARGO_HOME")
    configured = matches[0]
    expanded = Path(str(resolved_cargo_home(environment)) + configured[len("$CARGO_HOME"):])
    if not expanded.is_absolute() or "$" in str(expanded):
        raise ValueError("deny.toml advisory db-path did not expand to an absolute path")
    return expanded


def database_identity(root: Path) -> dict[str, str]:
    repositories = sorted(path.parent for path in root.rglob(".git") if path.is_dir())
    if root.joinpath(".git").is_dir():
        repositories.insert(0, root)
    repositories = list(dict.fromkeys(repositories))
    matches: list[dict[str, str]] = []
    for repository in repositories:
        remote = subprocess.run(
            ["/usr/bin/git", "-C", str(repository), "config", "--get", "remote.origin.url"],
            check=True, capture_output=True, text=True,
        ).stdout.strip().removesuffix(".git")
        if remote.casefold() != REPOSITORY.casefold():
            continue
        revision = subprocess.run(
            ["/usr/bin/git", "-C", str(repository), "rev-parse", "HEAD"],
            check=True, capture_output=True, text=True,
        ).stdout.strip()
        if len(revision) != 40 or any(character not in "0123456789abcdef" for character in revision):
            raise ValueError("advisory database HEAD is not a full Git revision")
        matches.append({"repository": REPOSITORY, "revision": revision})
    if len(matches) != 1:
        raise OSError(f"expected one RustSec advisory database under {root}, found {len(matches)}")
    return matches[0]


def main() -> int:
    try:
        child_environment = dict(os.environ)
        child_environment["CARGO_HOME"] = str(resolved_cargo_home(child_environment))
        root = database_root(environment=child_environment)
        result = subprocess.run(
            ["cargo", "deny", "check", "advisories"],
            check=False, env=child_environment,
        )
        if result.returncode != 0:
            return result.returncode
        identity = database_identity(root)
        output_path = os.environ.get("TL_ADVISORY_IDENTITY_OUTPUT")
        if output_path:
            Path(output_path).write_text(
                json.dumps(identity, indent=2, sort_keys=True) + "\n", encoding="utf-8"
            )
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"cannot identify the advisory database used by cargo-deny: {error}", file=sys.stderr)
        return 2
    print(f"advisories ok at {identity['revision']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
