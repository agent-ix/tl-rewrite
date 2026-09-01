#!/usr/bin/env bash
set -euo pipefail

cargo doc --no-deps --all-features
target_dir="${CARGO_TARGET_DIR:-target}"
index="${target_dir}/doc/tl_rewrite/index.html"
if [[ ! -s "$index" ]]; then
  echo "rustdoc did not produce a non-empty tl_rewrite index" >&2
  exit 1
fi
if ! /usr/bin/grep -Fq 'tl_rewrite' "$index"; then
  echo "rustdoc index does not identify the tl_rewrite crate" >&2
  exit 1
fi
digest="$(/usr/bin/sha256sum "$index" | /usr/bin/cut -d' ' -f1)"
/usr/bin/printf 'observed rustdoc index SHA-256 %s\n' "$digest"
