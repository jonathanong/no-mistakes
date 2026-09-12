---
type: llm
focus: last_message
weight: 1
---

Passing: the plan recognises this is a type-flow change — every
site that consumes the RESULT in a boolean position is affected,
not just the call sites — and covers consumers across packages
reached via the package entrypoint.

Failing: the plan enumerates callers only, with no attention to
how the return value is consumed.
