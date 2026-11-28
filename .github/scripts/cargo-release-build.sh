#!/usr/bin/env bash
# Build the CLI binary (no-mistakes) and Node N-API addon (no-mistakes-napi) in
# one Cargo invocation so no-mistakes rlib compiles once. --timings writes
# target/cargo-timings/cargo-timing-<stamp>.html next to that compile.
set -euo pipefail

target="${NM_TARGET:?Set NM_TARGET}"

free -h || true
if [ "${RUNNER_OS:-}" = Linux ] && [ -x /usr/bin/time ]; then
  /usr/bin/time -v cargo build --release --locked --target "$target" -p no-mistakes -p no-mistakes-napi --timings
else
  cargo build --release --locked --target "$target" -p no-mistakes -p no-mistakes-napi --timings
fi
shopt -s nullglob
timing_reports=(target/cargo-timings/cargo-timing-*.html)
if [ "${#timing_reports[@]}" -eq 0 ]; then
  echo "cargo --timings did not write target/cargo-timings/cargo-timing-*.html" >&2
  exit 1
fi
free -h || true
