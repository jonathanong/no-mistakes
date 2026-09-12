---
type: llm
focus: last_message
weight: 1
---

Passing: the plan enumerates actual CALL SITES (not merely files
that import the symbol), and covers consumers reached through the
package entrypoint rather than a relative path.

Failing: the plan stops at importing files, or scopes the search
to the defining package.
