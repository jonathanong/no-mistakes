---
type: llm
focus: last_message
weight: 1
---

Passing: the plan resolves whether the symbol is re-exported
through the package entrypoint (and therefore reachable by other
packages), rather than judging by where the file sits or whether
the declaration says `export`.

Failing: the plan concludes from the `export` keyword alone, or
from directory location.
