#!<SYSTEM>/bin/env bash
set -euo pipefail
# Configure the documented stable toolchain on PATH before running.
label="$1"
evidence_dir=<WORKSPACE>/performance-1972-evidence
rustc +stable --edition=2021 -C opt-level=3 -C lto=true -C codegen-units=1 \
  -L dependency=<WORKSPACE>/performance-1972-target/release/deps \
  --extern "no_mistakes=$evidence_dir<SYSTEM_LIB>$label.rlib" \
  "$evidence_dir/facts-probe.rs" -o "$evidence_dir/$label-facts"
sha256sum "$evidence_dir/$label-facts"
