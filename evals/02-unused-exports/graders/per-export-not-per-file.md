---
type: llm
focus: last_message
weight: 1
---

The plan evaluates liveness for each exported symbol individually, not
for the file as a whole.

Passing: makes clear that a file can be imported while specific exports
within it have no consumers, and proposes a per-symbol check.

Failing: decides the question by asking only whether the file is
imported anywhere, or checks the module as a single unit.
