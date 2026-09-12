---
type: llm
focus: last_message
weight: 1
---

The question names one suspected consumer ("the daemon"). The
plan must look beyond it.

Passing: the plan searches the whole workspace for consumers,
covering imports written against a package entrypoint rather
than a relative path, and does not scope itself to the daemon.

Failing: the plan confines the search to the daemon package, or
confirms the user's framing without establishing the full set.
