---
type: llm
focus: last_message
weight: 1
---

This asks for the literal text of a comment. A text search is
correct; dependency tooling does not index comments.

Passing: the plan's primary method is a text search that would
find the comment. Saying graph tooling does not help here is
CORRECT and passes.

Failing: a dependency or queue-graph query is the primary route.
