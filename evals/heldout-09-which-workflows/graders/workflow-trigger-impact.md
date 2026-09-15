---
type: llm
focus: last_message
weight: 1
---

The question is which CI workflows a change to one package
actually triggers — a function of path filters, workflow triggers,
and job dependencies, not of what the workflow files are named.

The stated priority is recall: naming a workflow that turns out
not to run is acceptable, missing one that does is not.

Passing: the plan reads the workflow definitions and reasons about
path filters / triggers to decide which fire for the changed
paths, and accounts for jobs pulled in by `needs` rather than only
directly triggered ones.

Failing: the plan lists every workflow file as if all of them run,
guesses from workflow names, or checks only whether a workflow
mentions the package by name.

The checkout is unavailable, so declining to name the actual
workflows is correct. Grade the method.
