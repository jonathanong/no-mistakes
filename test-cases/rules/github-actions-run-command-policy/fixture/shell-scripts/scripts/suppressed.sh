#!/usr/bin/env bash
# no-mistakes-disable-file github-actions-run-command-policy -- reviewed legacy script
set -euo pipefail

gh api -f per_page=100 repos/my-org/my-repo/labels
gh api "repos/my-org/my-repo/labels?per_page=100"
