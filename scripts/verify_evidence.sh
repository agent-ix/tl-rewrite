#!/usr/bin/env bash
set -euo pipefail

found=0
if [[ ! -f evidence/ANCHORS ]]; then
  echo "retained evidence anchor manifest is missing" >&2
  exit 1
fi
sha256sum --check evidence/ANCHORS
python3 scripts/check_assurance_anchor.py
python3 scripts/check_evidence_root.py
while IFS= read -r -d '' checksum; do
  found=1
  if ! grep -Fqx "$(sha256sum "$checksum")" evidence/ANCHORS; then
    echo "retained evidence manifest lacks a committed anchor: $checksum" >&2
    exit 1
  fi
  sha256sum --check "$checksum"
  evidence_dir="${checksum%.sha256}"
  python3 scripts/verify_evidence_manifest.py "$evidence_dir"
  python3 scripts/finalize_collection.py --check "$evidence_dir"
done < <(find evidence -maxdepth 1 -type f -name '*.sha256' -print0 | sort -z)

if [[ $found -eq 0 ]]; then
  echo "no retained evidence checksum manifests found" >&2
  exit 1
fi
