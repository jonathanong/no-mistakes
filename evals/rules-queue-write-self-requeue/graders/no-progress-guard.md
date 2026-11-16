---
type: llm
focus: last_message
weight: 1
---

Grade only the code in the final message.

Passing: before re-enqueuing itself the job checks that
the batch made progress (it counts rows it actually
filled, or advances a cursor, and stops or fails when a
batch fills none), so rows that cannot be filled (an
empty slug) cannot make the job re-enqueue forever.

Failing: the job re-enqueues whenever it read a full
batch of 500 (or any non-empty batch) with no check that
progress was made.

General advice, caveats, or lists of best practices earn nothing; grade the code only.
