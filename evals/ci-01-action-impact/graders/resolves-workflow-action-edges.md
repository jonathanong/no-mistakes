---
type: llm
focus: last_message
weight: 1
---

Passing: the plan determines which workflow files reference the
composite action — including indirectly, where one workflow calls
another that uses it — rather than assuming only the obviously
named workflow.

Failing: the plan guesses from workflow filenames, or checks only
for a literal path string without considering reusable-workflow
indirection.
