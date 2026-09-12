---
type: llm
focus: last_message
weight: 1
---

Passing: the plan verifies the answer against the type
declarations and/or a fixture comparison of both outputs, rather
than assuming parity.

Failing: the plan asserts they match (or differ) with no
verification step.
