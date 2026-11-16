---
type: llm
focus: last_message
weight: 1
---

Grade only the workflow in the final message, and only
for its checkout step(s).

Passing: every `actions/checkout` step sets
`persist-credentials: false`.

Failing: any `actions/checkout` step omits
`persist-credentials` or sets it to true.

General advice, caveats, or lists of best practices earn nothing; grade the code only.
