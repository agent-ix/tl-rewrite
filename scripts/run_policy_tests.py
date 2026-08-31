#!/usr/bin/env python3
"""Run every checked-in evidence-policy behavior test by census."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def main() -> int:
    tests = sorted((ROOT / "scripts").glob("test_*.py"))
    if not tests:
        print("no evidence-policy behavior tests found", file=sys.stderr)
        return 1
    for test in tests:
        result = subprocess.run([sys.executable, str(test)], cwd=ROOT, check=False)
        if result.returncode != 0:
            return result.returncode
    print(f"all {len(tests)} evidence-policy behavior tests passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
