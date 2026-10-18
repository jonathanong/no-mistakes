#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
output=${1:?output directory required}
manifest="$root/crates/no-mistakes/src/lib.rs"
host=$(rustc -vV | sed -n 's/^host: //p')
case "$host" in
  aarch64-unknown-linux-gnu) lto=fat; units=1 ;;
  aarch64-apple-darwin) lto=thin; units=16 ;;
  *) echo "unsupported native runner target: $host" >&2; exit 2 ;;
esac
backup="$manifest.nm2120-backup"
test ! -e "$backup"
cp "$manifest" "$backup"
restore() {
  mv "$backup" "$manifest"
  # Make Cargo rebuild after the temporary negative-state source mutation.
  touch "$manifest"
}
trap restore EXIT
mkdir -p "$output"
run_probe() {
  label=$1
  shift
  CARGO_TARGET_DIR="$output/target" CARGO_BUILD_JOBS=2 CARGO_PROFILE_RELEASE_LTO="$lto" \
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS="$units" \
    CARGO_PROFILE_RELEASE_DEBUG=0 CARGO_INCREMENTAL=0 RUSTC_WRAPPER='' \
    cargo run --release --locked --example allocator_probe "$@" > "$output/$label.log" 2>&1
}
run_probe feature-off --no-default-features --features mermaid-validation
python3 - "$manifest" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
block = '#[cfg(feature = "mimalloc")]\n#[global_allocator]\nstatic GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;\n'
if text.count(block) != 1:
    raise SystemExit("global allocator probe block was missing or duplicated")
path.write_text(text.replace(block, "", 1))
PY
if run_probe missing-registration --features mimalloc; then
  echo "allocator probe unexpectedly passed without global registration" >&2
  exit 1
fi
grep -Fq 'global allocator wiring mismatch' "$output/missing-registration.log"
restore
trap - EXIT
run_probe registered --features mimalloc
cat "$output/feature-off.log" "$output/missing-registration.log" "$output/registered.log"
