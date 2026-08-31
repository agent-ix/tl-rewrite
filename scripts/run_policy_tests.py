#!/usr/bin/env python3
"""Run every checked-in evidence-policy behavior test by census."""

from __future__ import annotations

import subprocess
import os
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
REQUIRED = {
    "test_evidence_root.py", "test_evidence_tool.py", "test_failure_propagation.py",
    "test_json_schema_gate.py", "test_traceability_gate.py",
}


def main() -> int:
    if sys.flags.optimize or os.environ.get("PYTHONOPTIMIZE"):
        print("optimized Python disables policy assertions", file=sys.stderr)
        return 2
    if len(sys.argv) == 1:
        directory = ROOT / "scripts"
        required = REQUIRED
    elif len(sys.argv) == 3 and sys.argv[1] == "--directory":
        directory = Path(sys.argv[2])
        required = None
    else:
        return 2
    tests = sorted(directory.glob("test_*.py"))
    observed = {test.name for test in tests}
    if not tests or (required is not None and observed != required):
        print(f"policy test census drift: missing={sorted((required or set()) - observed)}, extra={sorted(observed - (required or observed))}", file=sys.stderr)
        return 1
    child_env = dict(os.environ)
    child_env.pop("PYTHONOPTIMIZE", None)
    for test in tests:
        result = subprocess.run([sys.executable, str(test)], cwd=ROOT, check=False, env=child_env)
        if result.returncode != 0:
            return result.returncode
    print(f"all {len(tests)} evidence-policy behavior tests passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
