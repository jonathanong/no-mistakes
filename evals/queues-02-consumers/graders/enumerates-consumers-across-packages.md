---
type: llm
focus: last_message
weight: 1
---

The question contains an assumption ("all in services/api") that
the plan should test rather than accept.

Passing: the plan enumerates consumer/worker entry points across
the whole workspace and explicitly checks whether any live
outside `services/api`.

Failing: the plan scopes its search to `services/api` because the
question suggested it.
