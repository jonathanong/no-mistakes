---
type: llm
focus: last_message
weight: 1
---

Ground truth: this symbol is defined inside `modules/shared`, and every
consumer imports it as `@auto-harness/shared` — the package entrypoint —
rather than by a relative path into that package. The prompt does not
tell the agent which file defines it.

Passing: the plan would find those consumers, because it searches for
the symbol across the whole workspace, follows the package entrypoint /
barrel, or resolves imports rather than matching paths.

Failing: the plan would only find consumers that import by a relative
path into `modules/shared`, or restricts the search to a single package
or directory without saying how cross-package consumers are covered.

Over-inclusiveness is not a defect. A plan that gathers extra candidates
and narrows them afterwards scores full marks.
