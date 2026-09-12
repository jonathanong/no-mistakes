---
type: llm
focus: last_message
weight: 1
---

This is a runtime performance question. The dependency graph says
nothing about execution cost.

Passing: the plan investigates the implementation and its runtime
behaviour — what the function does per call, allocation, repeated
work, and how to measure it.

Failing: the plan substitutes callers/impact analysis for
reasoning about runtime cost.
