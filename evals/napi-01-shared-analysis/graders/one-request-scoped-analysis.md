---
type: llm
focus: last_message
weight: 1
---

The agent cannot read the implementation, so hedging on whether
the addon actually rebuilds ("wasteful IF each call rebuilds the
graph") is CORRECT and must not be penalised.

Passing: the answer identifies that the expensive shared work is
building the analysis/graph — not the call overhead — and either
recommends a single shared request-scoped analysis producing all
three reports, or names checking for one as the next step.

Failing: the answer treats the cost as N-API boundary crossings
or serialization, optimises by caching results outside the API,
or never identifies duplicated graph construction as the issue.
