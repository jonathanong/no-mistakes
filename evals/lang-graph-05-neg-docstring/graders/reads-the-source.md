---
type: llm
focus: last_message
weight: 1
---

This asks for literal docstring text. Reading the file is
correct.

Passing: the plan reads the function's source to quote it.
Noting that graph tooling does not index docstrings is CORRECT
and passes.

Failing: a dependency query is the primary route.
