---
type: llm
focus: last_message
weight: 1
---

The response gives a concrete way to obtain the NARROWED list of tests
that actually reach `modules/shared/src/authz.ts` — not just an
explanation of why the full suite runs.

Passing: names a specific command, planner, or explicit step-by-step
procedure that would yield the reduced list of test files.

Failing: stops at diagnosis, or only proposes configuration changes
(splitting the barrel, changing CI filters) with no way to compute which
tests actually reach the change.
