#!/usr/bin/env bash
set -euo pipefail

gh api -f per_page=100 repos/my-org/my-repo/labels
gh api "repos/my-org/my-repo/labels?per_page=100"
gh api -f x=1 -H "-X: custom" repos/my-org/my-repo/labels
gh api --method= -f x=1 repos/my-org/my-repo/labels
gh api graphql -f query='query { viewer { login } }'
gh api --method GET -f per_page=100 repos/my-org/my-repo/labels
gh api -f x=1 "$ENDPOINT"
gh api --method "$METHOD" -f x=1 repos/my-org/my-repo/labels
# no-mistakes-disable-next-line github-actions-run-command-policy -- reviewed fixture
gh api -f x=1 repos/my-org/my-repo/labels
gh api "user?per_page=100" # no-mistakes-disable-line github-actions-run-command-policy -- reviewed fixture
