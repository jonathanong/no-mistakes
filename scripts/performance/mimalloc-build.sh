#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
variant=${1:?variant must be system or mimalloc}
out=${2:?output directory required}
case "$variant" in system|mimalloc) ;; *) exit 2 ;; esac
source "$root/scripts/performance/mimalloc-build-args.sh"
target=$(rustc -vV | sed -n 's/^host: //p')
case "$target" in
  x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu) lto=fat; units=1; addon=libno_mistakes.so ;;
  aarch64-apple-darwin) lto=thin; units=16; addon=libno_mistakes.dylib ;;
  *) echo "unsupported native runner target: $target" >&2; exit 2 ;;
esac
manifest="$root/crates/no-mistakes/Cargo.toml"
backup="$manifest.nm2120-backup"
test ! -e "$backup"
cp "$manifest" "$backup"
restore() { mv "$backup" "$manifest"; }
trap restore EXIT
python3 - "$manifest" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
old = 'crate-type = ["rlib"]'
if text.count(old) != 1:
    raise SystemExit(f"expected exactly one {old!r} in {path}")
path.write_text(text.replace(old, 'crate-type = ["rlib", "cdylib"]', 1))
PY
mkdir -p "$out/$variant"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$out/target-$variant}"
export CARGO_PROFILE_RELEASE_LTO="$lto"
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS="$units"
export CARGO_PROFILE_RELEASE_DEBUG=0
export CARGO_INCREMENTAL=0
export RUSTC_WRAPPER=
export NO_MISTAKES_BUILD_NAPI=1
# Same artifacts as release.yml: one rustc emits the CLI rlib and the addon.
# Leave CARGO_BUILD_JOBS unset so Cargo uses its default, matching release.yml.
mimalloc_build "$variant" cargo build --release --locked --timings --target "$target" \
  -p no-mistakes --lib --bin no-mistakes
binary="$CARGO_TARGET_DIR/$target/release/no-mistakes"
cp "$binary" "$out/$variant/no-mistakes"
cp "$CARGO_TARGET_DIR/$target/release/$addon" "$out/$variant/$addon"
cp "$out/$variant/$addon" "$out/$variant/no-mistakes.node"
if [[ "$target" == *-unknown-linux-gnu ]]; then
  python3 "$root/scripts/performance/mimalloc-tls.py" "$out/$variant/no-mistakes.node" "$out/$variant"
fi
# The probe only checks registration. Build it separately with LTO off so
# dev-dependency features are not unified into the measured CLI and addon.
probe_target="$out/probe-target-$variant"
CARGO_TARGET_DIR="$probe_target" CARGO_PROFILE_RELEASE_LTO=off CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 \
  mimalloc_build "$variant" cargo build --release --locked --target "$target" \
  -p no-mistakes --example allocator_probe
cp "$probe_target/$target/release/examples/allocator_probe" "$out/$variant/rayon-probe"
python3 - "$out/$variant" <<'PY'
import hashlib, pathlib, sys
root = pathlib.Path(sys.argv[1])
with (root / "SHA256SUMS").open("w") as sums:
    for name in ("no-mistakes", "no-mistakes.node", "rayon-probe"):
        digest = hashlib.sha256((root / name).read_bytes()).hexdigest()
        sums.write(f"{digest}  {name}\n")
PY
python3 - "$root" "$out/$variant/provenance.json" "$variant" "$target" <<'PY'
import hashlib, json, os, platform, subprocess, sys
root, output, variant, target = sys.argv[1:]
def run(*args): return subprocess.check_output(args, cwd=root, text=True).strip()
def sha256(path):
    with open(os.path.join(root, path), "rb") as handle:
        return hashlib.sha256(handle.read()).hexdigest()
provenance = {
    "variant": variant, "source": run("git", "rev-parse", "HEAD"),
    "tree": run("git", "rev-parse", "HEAD^{tree}"), "target": target,
    "lock": {"path": "Cargo.lock", "sha256": sha256("Cargo.lock")},
    "rustc": run("rustc", "-Vv"), "cargo": run("cargo", "-V"),
    "features": "default" if variant == "mimalloc" else "--no-default-features --features mermaid-validation",
    "lto": os.environ["CARGO_PROFILE_RELEASE_LTO"],
    "codegenUnits": int(os.environ["CARGO_PROFILE_RELEASE_CODEGEN_UNITS"]),
    "build_flags": {key: os.environ.get(key) for key in (
        "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "CARGO_PROFILE_RELEASE_DEBUG",
        "CARGO_INCREMENTAL", "NO_MISTAKES_BUILD_NAPI")},
    "runnerOS": os.environ.get("RUNNER_OS"), "imageOS": os.environ.get("ImageOS"),
    "imageVersion": os.environ.get("ImageVersion"), "machine": platform.machine(),
}
for name in ("no-mistakes", "no-mistakes.node", "rayon-probe"):
    provenance[name] = {"bytes": os.path.getsize(os.path.join(os.path.dirname(output), name)),
                        "sha256": hashlib.sha256(open(os.path.join(os.path.dirname(output), name), "rb").read()).hexdigest()}
# Record Cargo's job cap only when the environment actually set it.
jobs = os.environ.get("CARGO_BUILD_JOBS")
if jobs and jobs.isdigit() and int(jobs) > 0:
    provenance["buildJobs"] = int(jobs)
open(output, "w").write(json.dumps(provenance, indent=2) + "\n")
PY
