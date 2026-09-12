---
type: llm
focus: last_message
weight: 1
---

The response explains WHY the whole suite is being selected, in terms of
the dependency structure — for example that `modules/shared` sits
upstream of everything, or that the single barrel entrypoint puts the
changed file in every package's module graph.

Passing: gives a structural reason grounded in the import graph.

Failing: offers no explanation, or attributes it only to configuration
without reference to the dependency structure.
