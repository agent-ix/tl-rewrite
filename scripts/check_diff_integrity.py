#!/usr/bin/env python3
"""Run the exact candidate diff check and emit a positive completion signature."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check_diff_integrity.py SOURCE_REVISION", file=sys.stderr)
        return 2
    revision = sys.argv[1]
    result = subprocess.run(
        ["/usr/bin/git", "diff", "--check", f"origin/main...{revision}"],
        cwd=ROOT,
        check=False,
    )
    if result.returncode != 0:
        return result.returncode
    print(f"diff-integrity gate passed for origin/main...{revision}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
