---
type: llm
focus: last_message
weight: 1
---

Passing: the plan asks for the COMPLETE local validation set for
the changed files — impacted tests plus the other configured
checks (lint, typecheck, and any repository-configured rules) —
rather than tests alone.

Failing: the plan proposes only running tests, or only a generic
"run the test suite and lint" without tying either to the files
that changed.
