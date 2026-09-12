---
type: llm
focus: last_message
weight: 1
---

This is a lookup in `package.json`. Reading that file is correct.

Passing: the plan reads or searches `package.json` for the
script name. Noting that graph tooling is not relevant here is
CORRECT and passes.

Failing: a dependency or impact query is the primary route.
