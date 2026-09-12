---
type: llm
focus: last_message
weight: 1
---

Passing: the plan establishes what each package actually
contains and compares the two — by any method, including leading
with tooling — and considers whether either already consumes a
shared implementation from `@auto-harness/shared` instead of
rolling its own. The comparison must be reachable from the plan;
it need not be the first step.

Failing: the plan inspects only one package, or answers from the
directory names with no step that would compare them.
