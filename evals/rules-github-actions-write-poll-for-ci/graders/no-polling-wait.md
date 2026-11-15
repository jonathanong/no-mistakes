---
type: llm
focus: last_message
weight: 1
---

Grade only the workflow in the final message.

Passing: the publish waits on CI through workflow
structure: it calls ci.yml as a reusable workflow and
depends on it with `needs`, or uses a `workflow_run`
trigger with validation, or an equivalent event-driven
mechanism. No step waits by looping.

Failing: any step polls for CI status with a sleep loop,
`gh run watch`, `gh run list`/`gh api` in a retry loop,
or a third-party "wait for checks" action. Also failing:
a one-shot lookup of CI's result through `gh` or the API,
because on a tag push CI for that commit may still be
running, so the gate races it.

General advice, caveats, or lists of best practices earn nothing; grade the code only.
