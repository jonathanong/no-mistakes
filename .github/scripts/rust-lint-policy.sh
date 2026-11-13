#!/usr/bin/env bash
set -euo pipefail

lint_policy_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
lint_policy_temp=$(mktemp -d)
trap 'rm -rf -- "$lint_policy_temp"' EXIT
cp -R -- "$lint_policy_root/fixtures/cargo-lint-policy/fixture" "$lint_policy_temp/source"

env -u CARGO_ENCODED_RUSTFLAGS \
  CLIPPY_CONF_DIR="$lint_policy_root" \
  CARGO_TARGET_DIR="$lint_policy_temp/target" \
  CARGO_BUILD_JOBS=1 RUSTC_WRAPPER= RUSTFLAGS="-D warnings" \
  cargo clippy --offline --manifest-path "$lint_policy_temp/source/Cargo.toml" -- -D warnings
