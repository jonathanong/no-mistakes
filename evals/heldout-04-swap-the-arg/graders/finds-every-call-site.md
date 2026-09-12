---
type: llm
focus: last_message
weight: 1
---

Reshaping a parameter list breaks CALL SITES, not merely the
files that import the symbol.

Passing: the plan commits to enumerating actual call sites across
the whole workspace, including consumers that import through the
package entrypoint rather than a relative path, before saying
anything about how large the change is.

Failing: the plan stops at the set of importing files, scopes the
search to the defining package, or offers a size estimate not
grounded in a method that finds every caller.

The checkout is unavailable, so declining to state an actual
count is correct. Grade the method, not whether a number appears.
