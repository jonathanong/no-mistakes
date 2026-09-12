---
type: llm
focus: last_message
weight: 1
---

This is a literal value lookup in one known file. Reading it is
correct.

Passing: the plan reads `ci.yml` to report the value. Noting that
graph tooling is unnecessary is CORRECT and passes.

Failing: a CI-impact or topology query is the primary route.
