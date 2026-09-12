---
type: llm
focus: last_message
weight: 1
---

This is a design-judgement question, not a structural query.

Passing: the plan engages with the trade-off — shared state,
testability, how the helpers are actually consumed, and what
would change at the call sites.

Failing: the plan answers with an impact/dependency query in
place of a design argument.

Using the consumer set as *evidence* for a design argument is
fine; substituting it for the argument is not.
