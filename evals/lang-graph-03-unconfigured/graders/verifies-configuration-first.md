---
type: llm
focus: last_message
weight: 1
---

The trap is concluding "no results, therefore no usages" for a
language that was never configured.

Passing: the plan checks the configuration to confirm whether
Ruby is set up BEFORE interpreting any empty result, and says an
empty result from an unconfigured language means nothing.

Failing: the plan runs a query and reads an empty result as
evidence about the code.
