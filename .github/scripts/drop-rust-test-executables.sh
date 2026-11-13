#!/usr/bin/env bash
set -euo pipefail

target=${1:-${CARGO_TARGET_DIR:-target}}
profile_dir="$target/debug"
found=0
dropped=0
for dir in "$profile_dir/deps" "$profile_dir/build"; do
  [[ -d "$dir" ]] || continue
  found=1
  count=$(find "$dir" -type f -perm -u+x ! -name '*.*' ! -name 'build_script_*' ! -name 'build-script-*' -print -delete | wc -l)
  dropped=$((dropped + count))
done
if [[ $found == 0 ]]; then
  echo "::error::No Cargo artifact directory under $profile_dir (expected deps/ or build/)." >&2
  exit 1
fi
echo "Dropped $dropped Rust test executables from $profile_dir"
