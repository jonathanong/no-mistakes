#!/usr/bin/env bash
set -euo pipefail

version="$1"
push_status=0
output="$(LC_ALL=C git push --atomic --porcelain origin HEAD:main "v$version" 2>&1)" || push_status=$?
if [[ "$push_status" -eq 0 ]]; then
  printf '%s\n' "$output"
  exit 0
fi

printf '%s\n' "$output" >&2
if grep -Eq \
  -e '^fatal: .*returned error: 403([[:space:]]|$)' \
  -e '^remote: Permission .*denied' \
  -e '^(fatal|remote): Authentication failed' \
  -e '^remote: error: GH006: Protected branch update failed for refs/heads/main\.$' \
  -e $'^!\t[^\t]+:refs/heads/main\t\\[remote rejected\\] \\(protected branch hook declined\\)$' \
  -e '^remote: error: GH013: Repository rule violations found for refs/heads/main\.' \
  <<< "$output"; then
  echo "::error::The release GitHub App could not push the release commit and tag. Verify that the App is installed on this repository with Contents write access, is a bypass actor on main's ruleset, and is allowed to bypass main's branch protection rule." >&2
elif grep -Eq $'^!\t[^\t]+:refs/heads/main\t\\[rejected\\] \\((non-fast-forward|fetch first)\\)$' <<< "$output"; then
  echo "::error::main moved while the release commit was being prepared. Re-run the release workflow from current main." >&2
fi
exit "$push_status"
