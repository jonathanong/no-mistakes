#!/usr/bin/env bash
set -euo pipefail

case "${1:-}" in
  verify)
    if [[ ! "${BENCHMARK_SHA256:-}" =~ ^[0-9a-f]{64}$ ]]; then
      echo "Missing or invalid benchmark digest from the build job." >&2
      exit 1
    fi
    if ! printf '%s  %s\n' "$BENCHMARK_SHA256" "${2:?archive path required}" | sha256sum --check --status; then
      echo "Benchmark archive does not match the build job's digest; refusing to extract or execute it." >&2
      exit 1
    fi
    ;;
  available)
    # Official cache/save can warn and succeed without actually saving a cache.
    if [[ "${CACHE_HIT:-}" != true && ! "${ARTIFACT_ID:-}" =~ ^[1-9][0-9]*$ ]]; then
      echo "No exact benchmark cache or uploaded fallback artifact is available. Rerun the build job." >&2
      exit 1
    fi
    ;;
  *)
    echo "Usage: benchmark-handoff.sh verify ARCHIVE | available" >&2
    exit 2
    ;;
esac
