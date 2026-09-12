---
type: llm
focus: last_message
weight: 1
---

Passing: the plan queries BOTH configured language graphs
separately and unions the results, recognising that one graph
will not return the other language's consumers.

Failing: the plan runs a single query and treats it as covering
both languages.
