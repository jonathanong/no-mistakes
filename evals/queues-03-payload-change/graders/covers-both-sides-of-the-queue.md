---
type: llm
focus: last_message
weight: 1
---

Passing: the plan covers BOTH the producing side and the
consuming side, and notes that because they are decoupled by a
queue, a payload change will NOT necessarily surface as a
compile/type error — in-flight or persisted messages in the old
shape are a risk the plan should raise.

Failing: the plan treats this as an ordinary type-change blast
radius that the type checker would catch.
