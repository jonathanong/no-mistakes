---
type: llm
focus: last_message
weight: 1
---

Passing: the plan recommends the in-process programmatic API and
explains the reason in terms of shared work — one request-scoped
analysis reused across queries, instead of rebuilding the graph
and paying process startup per file.

Failing: the plan only suggests parallelising, batching, or
caching the subprocess calls, without reaching for the in-process
API.
