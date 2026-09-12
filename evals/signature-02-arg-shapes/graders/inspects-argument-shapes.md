---
type: llm
focus: last_message
weight: 1
---

The question is about the ARGUMENTS passed, not merely where the
function is called.

Passing: the plan gathers the actual argument values/shapes at
each call site so the enum's members can be derived from real
usage.

Failing: the plan only locates call sites and leaves the argument
inspection unaddressed.
