#!/usr/bin/env bash
# One rustc emits the CLI rlib and the Node cdylib. --timings writes
# target/cargo-timing.html next to that compile.
set -euo pipefail

target="${NM_TARGET:?Set NM_TARGET}"

python - <<'PY'
from pathlib import Path
path = Path("crates/no-mistakes/Cargo.toml")
text = path.read_text()
old = 'crate-type = ["rlib"]'
new = 'crate-type = ["rlib", "cdylib"]'
if old not in text:
    raise SystemExit(f"expected {old!r} in {path}")
path.write_text(text.replace(old, new, 1))
PY

free -h || true
if [ "${RUNNER_OS:-}" = Linux ] && [ -x /usr/bin/time ]; then
  /usr/bin/time -v env NO_MISTAKES_BUILD_NAPI=1 cargo build --release --locked --target "$target" -p no-mistakes --timings
else
  NO_MISTAKES_BUILD_NAPI=1 cargo build --release --locked --target "$target" -p no-mistakes --timings
fi
free -h || true
