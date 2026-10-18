#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
cd "$root"
out=${1:?output directory required}
target=$(rustc -vV | sed -n 's/^host: //p')
binary=no-mistakes
case "$target" in
  aarch64-apple-darwin) lto=thin; units=16; addon=libno_mistakes.dylib; magic='Mach-O.*arm64' ;;
  x86_64-unknown-linux-gnu) lto=fat; units=1; addon=libno_mistakes.so; magic='ELF 64-bit.*x86-64' ;;
  aarch64-unknown-linux-gnu) lto=fat; units=1; addon=libno_mistakes.so; magic='ELF 64-bit.*ARM aarch64' ;;
  x86_64-pc-windows-msvc) lto=thin; units=16; addon=no_mistakes.dll; binary+=.exe; magic='PE32\+.*x86-64' ;;
  *) echo "unsupported release target: $target" >&2; exit 2 ;;
esac
test "$target" = "${EXPECTED_TARGET:?expected native target required}"
mkdir -p "$out"
manifest="$root/crates/no-mistakes/Cargo.toml"
backup="$manifest.mimalloc-release-backup"
test ! -e "$backup"
cp "$manifest" "$backup"
restore() { mv "$backup" "$manifest"; }
trap restore EXIT
# Match release.yml: one rustc emits the CLI's rlib and the Node cdylib.
python - "$manifest" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
old = 'crate-type = ["rlib"]'
if text.count(old) != 1:
    raise SystemExit(f"expected exactly one {old!r} in {path}")
path.write_text(text.replace(old, 'crate-type = ["rlib", "cdylib"]', 1))
PY
export CARGO_PROFILE_RELEASE_LTO="$lto"
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS="$units"
export CARGO_PROFILE_RELEASE_DEBUG=0
export CARGO_INCREMENTAL=0
export RUSTC_WRAPPER=
export NO_MISTAKES_BUILD_NAPI=1
export CARGO_TARGET_DIR
CARGO_TARGET_DIR=$(python -c 'from pathlib import Path; print(Path("target").resolve())')
# Explicitly require mimalloc while preserving all release default features.
cargo build --release --locked --target "$target" -p no-mistakes --features mimalloc 2>&1 | tee "$out/build.log"
release="target/$target/release"
file "$release/$binary" "$release/$addon" | tee "$out/file-types.txt"
file "$release/$binary" | grep -E "$magic"
file "$release/$addon" | grep -E "$magic"
cp "$release/$binary" "$out/$binary"
cp "$release/$addon" "$out/no-mistakes.node"
"$out/$binary" --version | tee "$out/cli-version.txt"
cargo tree --locked -p no-mistakes --features mimalloc -e features -i mimalloc > "$out/allocator-features.txt"
python - "$out" "$target" "$binary" <<'PY'
from pathlib import Path
import hashlib, json, os, platform, subprocess, sys
out, target, binary = Path(sys.argv[1]), sys.argv[2], sys.argv[3]
def run(*args):
    return subprocess.check_output(args, text=True).strip()
cpu = platform.processor() or os.environ.get("PROCESSOR_IDENTIFIER", "unavailable")
if Path("/proc/cpuinfo").exists():
    cpu = next((line.split(":", 1)[1].strip() for line in Path("/proc/cpuinfo").read_text().splitlines()
                if line.startswith("model name")), cpu)
artifacts = {}
for name in (binary, "no-mistakes.node"):
    path = out / name
    artifacts[name] = {"bytes": path.stat().st_size, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
data = {
    "source": run("git", "rev-parse", "HEAD"), "tree": run("git", "rev-parse", "HEAD^{tree}"),
    "target": target, "features": "default + mimalloc", "allocator": "mimalloc",
    "rustc": run("rustc", "-Vv"), "cargo": run("cargo", "-V"), "node": run("node", "--version"),
    "profile": {key: os.environ.get(key) for key in (
        "CARGO_PROFILE_RELEASE_LTO", "CARGO_PROFILE_RELEASE_CODEGEN_UNITS", "CARGO_PROFILE_RELEASE_DEBUG",
        "CARGO_INCREMENTAL", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "NO_MISTAKES_BUILD_NAPI")},
    "runner": {"os": os.environ.get("RUNNER_OS"), "image": os.environ.get("ImageOS"),
               "imageVersion": os.environ.get("ImageVersion"), "machine": platform.machine(),
               "cpu": cpu, "availableCpus": os.cpu_count()},
    "lockSha256": hashlib.sha256(Path("Cargo.lock").read_bytes()).hexdigest(), "artifacts": artifacts,
    "manifestRewrite": 'crate-type = ["rlib", "cdylib"]',
}
(out / "provenance.json").write_text(json.dumps(data, indent=2) + "\n")
with (out / "SHA256SUMS").open("w") as checksums:
    for name, facts in artifacts.items():
        checksums.write(f"{facts['sha256']}  {name}\n")
PY
