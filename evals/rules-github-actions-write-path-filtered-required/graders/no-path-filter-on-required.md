---
type: llm
focus: last_message
weight: 1
---

Grade only the workflow in the final message.

Passing: the workflow that produces the "backend / test"
check starts on every pull request to main (its trigger
has no `paths` or `paths-ignore` filter), and any
skipping of the tests for unrelated changes is done
inside the run, with a job that always reports.

Failing: `paths:` or `paths-ignore:` appears under the
`pull_request` trigger of that workflow.

General advice, caveats, or lists of best practices earn nothing; grade the code only.
