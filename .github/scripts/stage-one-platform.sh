#!/usr/bin/env bash
set -euo pipefail

version="${1:?version}"
package="${NM_PACKAGE:?Set NM_PACKAGE}"
target="${NM_TARGET:?Set NM_TARGET}"
runtime="${NM_RUNTIME:?Set NM_RUNTIME}"
extension="${NM_EXTENSION-}"

node packages/no-mistakes/scripts/stage-native.js \
  --package "$package" \
  --binary "dist/no-mistakes-v${version}-${target}${extension}" \
  --addon "dist/no-mistakes-napi-v${version}-${target}.node" \
  --platform "$runtime"
pnpm --filter "$package" pack --dry-run
