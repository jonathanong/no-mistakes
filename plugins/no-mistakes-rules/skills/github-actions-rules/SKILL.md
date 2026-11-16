---
name: github-actions-rules
description: "Load before writing, reviewing, or debugging anything under .github/workflows (ci.yml, release.yml, publish.yml), a composite action, a required check, or a CI failure. Lists the defects to avoid and the patterns that need a stated reason: pinning, permissions, untrusted input, timeouts, polling and gating on other workflows, required checks, concurrency, runners, CI log diagnosis."
---

# GitHub Actions rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

## Security

- MUST NOT reference actions by tag or branch; pin a full commit SHA with a version comment, and images by digest.
- MUST NOT expand `${{ }}` expressions in `run` or script bodies; pass them through `env`.
- MUST NOT check out or run pull-request code in a privileged `pull_request_target` or `workflow_run` workflow.
- MUST NOT act on `workflow_run` without validating the source workflow, repository, branch, head, and conclusion.
- MUST NOT grant write permissions at workflow level; grant least privilege per job.
- MUST NOT persist checkout credentials unless a later step pushes; set `persist-credentials: false`.
- MUST NOT download and run bytes without a pinned checksum.
- MUST NOT run untrusted code on persistent self-hosted runners.

## Reliability

- SHOULD NOT leave jobs without `timeout-minutes`, or set one at or above the platform cap.
- MUST NOT path-filter a required check; skipped jobs look missing. Use an always-reporting gate.
- MUST NOT fake required checks with check runs or statuses.
- MUST NOT rely on implicit success for jobs that must run after a failure; check `needs.<job>.result`.
- SHOULD NOT cancel in-progress runs on the default branch when each commit needs check results; the cancelled commit is left without them.
- MUST NOT use a local action before checking out its repository, or workflow-only keys in composite steps.
- MUST NOT assume runner-local state survives on ephemeral runners; use keyed caches.
- MUST NOT publish container ports to fixed host ports on hosted runners.
- MUST NOT hide stderr, quiet output, or add retries that mask a root cause.
- MUST NOT rerun CI to probe a flake; diagnose it.
- SHOULD NOT use large or extra-OS runners without a runtime reason.
- SHOULD NOT assert exact action SHAs or versions in tests; assert identity and SHA shape.
- SHOULD NOT parse workflow YAML with regexes; use a YAML parser.
