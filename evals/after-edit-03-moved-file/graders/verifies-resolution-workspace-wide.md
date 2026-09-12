---
type: llm
focus: last_message
weight: 1
---

Passing: the plan verifies that imports still RESOLVE across the
whole workspace, not just that the owning package compiles —
covering consumers in other packages and any path that referenced
the old location (including re-export/barrel lines).

Failing: the plan relies only on a local build or typecheck of
`modules/shared`, or only greps for the old path string.
