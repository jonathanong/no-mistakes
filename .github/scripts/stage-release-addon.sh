#!/usr/bin/env bash
set -euo pipefail

target="${NM_TARGET:?Set NM_TARGET}"
addon_name="${NM_ADDON:?Set NM_ADDON}"
version="${1:?version}"
addon="target/${target}/release/${addon_name}"
test -f "$addon"
case "$target" in
  aarch64-apple-darwin) expected_magic='Mach-O.*arm64' ;;
  aarch64-unknown-linux-gnu) expected_magic='ELF 64-bit.*ARM aarch64' ;;
  x86_64-unknown-linux-gnu) expected_magic='ELF 64-bit.*x86-64' ;;
  x86_64-pc-windows-msvc) expected_magic='PE32\+.*x86-64' ;;
  *) echo "::error::Unknown target ${target}" >&2; exit 1 ;;
esac
file "$addon" | grep -E "$expected_magic"
addon_artifact="no-mistakes-napi-v${version}-${target}.node"
cp "$addon" "$addon_artifact"
node -e '(async () => { const addon=require(process.argv[1]); if (typeof addon.version !== "function" || await addon.version() !== process.argv[2]) process.exit(1) })().catch((error) => { console.error(error); process.exit(1) })' "$PWD/$addon_artifact" "$version"
