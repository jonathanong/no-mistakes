#!/usr/bin/env bash
# Publish one package, then wait for its tarball. Callers run one package per
# job so registry confirmation overlaps instead of stacking.
set -euo pipefail

version="${1:?version}"
package="${2:?package}"
tools="${PUBLISH_TOOLS:-$RUNNER_TEMP/publish-tools}"

node "$tools/.github/scripts/publish-npm.cjs" "$package" "$version"
if ! node "$tools/packages/no-mistakes/scripts/wait-npm-tarball.js" --package "$package" --version "$version" --timeout-ms 2700000; then
  echo "$package@$version tarball verification failed. Inspect npm publication status and resume the same tag. See docs/development/release.md." >&2
  exit 1
fi
