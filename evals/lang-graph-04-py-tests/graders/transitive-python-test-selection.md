---
type: llm
focus: last_message
weight: 1
---

Passing: the plan selects tests that reach the file through the
Python import graph, including indirectly, rather than by
filename convention (`test_normalize.py`) alone.

Failing: test selection is by naming convention or directory
colocation only.
