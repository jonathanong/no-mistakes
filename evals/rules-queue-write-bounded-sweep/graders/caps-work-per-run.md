---
type: llm
focus: last_message
weight: 1
---

Grade only the code in the final message.

Passing: a single run is capped by rows, batches, or
elapsed time (a limit the loop actually enforces), so a
promo-sized backlog is left for a later run, and the
function reports or returns how much work remains or
that the cap was hit.

Failing: the function loops until a batch comes back
empty with no per-run cap, or runs one update over every
expired trial.

General advice, caveats, or lists of best practices earn nothing; grade the code only.
