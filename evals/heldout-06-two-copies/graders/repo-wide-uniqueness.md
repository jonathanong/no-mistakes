---
type: llm
focus: last_message
weight: 1
---

The question is whether a second implementation exists anywhere in
the repository — precisely what per-file linting cannot see.

The stated priority is recall: surfacing extra near-matches is
acceptable, missing a real duplicate is not.

Passing: the plan establishes uniqueness repository-wide, by
enumerating exported symbols across every package, rather than
inspecting one suspected location or relying on the asker's guess
about where a second copy would live.

Failing: the plan searches a single package or directory, or
treats "I only know of one" as evidence that only one exists.

The checkout is unavailable, so declining to say whether a
duplicate exists is correct. Grade the method.
