---
type: llm
focus: last_message
weight: 1
---

Grade only the review in the final message. Two checks.

1. Passing: the review flags the `paths:` filter on the
`pull_request` trigger as a problem for a required check
(a PR touching no listed path never runs the workflow, so
the required "backend / test" check is never reported and
the PR cannot merge), and asks for an always-reporting
gate or for the filter to be removed. Failing: the review
does not raise this.

2. Passing: the review does NOT demand changing
`persist-credentials: true` or the job-level
`contents: write` in the `snapshots` job. Failing: it
calls either a defect to fix.

Both checks must pass.

General advice, caveats, or lists of best practices earn nothing; grade the review's findings only.
