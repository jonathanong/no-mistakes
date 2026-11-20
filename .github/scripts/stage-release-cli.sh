#!/usr/bin/env bash
set -euo pipefail

target="${NM_TARGET:?Set NM_TARGET}"
version="${1:?version}"
case "$target" in
  *windows*) bin_ext=".exe" ;;
  *) bin_ext="" ;;
esac

mkdir -p release-assets
binary_source="target/${target}/release/no-mistakes${bin_ext}"
binary_artifact="release-assets/no-mistakes-v${version}-${target}${bin_ext}"
"$binary_source" --version
case "$target" in
  aarch64-apple-darwin) expected_magic='Mach-O.*arm64' ;;
  aarch64-unknown-linux-gnu) expected_magic='ELF 64-bit.*ARM aarch64' ;;
  x86_64-unknown-linux-gnu) expected_magic='ELF 64-bit.*x86-64' ;;
  x86_64-pc-windows-msvc) expected_magic='PE32\+.*x86-64' ;;
  *) echo "::error::Unknown target ${target}" >&2; exit 1 ;;
esac
file "$binary_source" | grep -E "$expected_magic"
cp "$binary_source" "$binary_artifact"
test -x "$binary_artifact" || [[ "$binary_artifact" == *.exe ]]
