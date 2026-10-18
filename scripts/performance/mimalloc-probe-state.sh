#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
state=${1:?state must be feature-off, missing-registration, or registered}
output=${2:?output directory required}
manifest="$root/crates/no-mistakes/src/lib.rs"
host=$(rustc -vV | sed -n 's/^host: //p')
case "$host" in
  aarch64-unknown-linux-gnu) lto=fat; units=1 ;;
  aarch64-apple-darwin) lto=thin; units=16 ;;
  *) echo "unsupported native runner target: $host" >&2; exit 2 ;;
esac
mkdir -p "$output"
backup="$manifest.nm2120-backup"
restore() {
  if [[ -e "$backup" ]]; then
    mv "$backup" "$manifest"
    touch "$manifest"
  fi
}
run_probe() {
  CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$output/target}" CARGO_BUILD_JOBS=2 \
    CARGO_PROFILE_RELEASE_LTO="$lto" CARGO_PROFILE_RELEASE_CODEGEN_UNITS="$units" \
    CARGO_PROFILE_RELEASE_DEBUG=0 CARGO_INCREMENTAL=0 RUSTC_WRAPPER='' \
    cargo run --release --locked --example allocator_probe "$@"
}
case "$state" in
  feature-off)
    run_probe --no-default-features --features mermaid-validation > "$output/probe.json" 2> "$output/probe.stderr"
    grep -Fq '"expected_mimalloc":false,"is_mimalloc_heap":false' "$output/probe.json"
    ;;
  registered)
    run_probe --features mimalloc > "$output/probe.json" 2> "$output/probe.stderr"
    grep -Fq '"expected_mimalloc":true,"is_mimalloc_heap":true' "$output/probe.json"
    ;;
  missing-registration)
    test ! -e "$backup"
    cp "$manifest" "$backup"
    trap restore EXIT
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
    if run_probe --features mimalloc > "$output/probe.stdout" 2> "$output/probe.stderr"; then
      echo "allocator probe unexpectedly passed without global registration" >&2
      exit 1
    fi
    grep -Fq 'global allocator wiring mismatch' "$output/probe.stderr"
    ;;
  *) echo "unsupported allocator probe state: $state" >&2; exit 2 ;;
esac
printf '{"state":"%s","target":"%s"}\n' "$state" "$host" > "$output/provenance.json"
