#!/usr/bin/env bash
# Publish one package, then wait for its tarball. Callers run one package per
# job so registry confirmation overlaps instead of stacking.
set -euo pipefail

version="${1:?version}"
package="${2:?package}"
wait_script="packages/no-mistakes/scripts/wait-npm-tarball.js"
tools="${PUBLISH_TOOLS:-$RUNNER_TEMP/publish-tools}"

if node "$wait_script" --package "$package" --version "$version" --timeout-ms 0 >/dev/null 2>&1; then
  echo "$package $version tarball is already fetchable; skipping publish"
else
  node "$tools/.github/scripts/publish-npm.cjs" "$package" "$version"
fi
node "$wait_script" --package "$package" --version "$version" --timeout-ms 1800000
