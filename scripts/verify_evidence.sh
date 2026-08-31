#!/usr/bin/env bash
set -euo pipefail

found=0
if [[ ! -f evidence/ANCHORS ]]; then
  echo "retained evidence anchor manifest is missing" >&2
  exit 1
fi
sha256sum --check evidence/ANCHORS
python3 scripts/check_assurance_anchor.py
while IFS= read -r -d '' record; do
  checksum="${record}.sha256"
  if [[ ! -f "$checksum" ]]; then
    if [[ -f "$record/.collecting" ]] && \
       ! git ls-files --error-unmatch "$record/.collecting" >/dev/null 2>&1; then
      continue
    fi
    echo "retained evidence directory lacks a checksum manifest: $record" >&2
    exit 1
  fi
done < <(find evidence -mindepth 1 -maxdepth 1 -type d -print0 | sort -z)
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
