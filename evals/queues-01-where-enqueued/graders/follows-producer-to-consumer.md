---
type: llm
focus: last_message
weight: 1
---

Producer and consumer here are decoupled by an outbox/queue —
the code that enqueues and the worker that drains share no
import edge, so following imports alone will not connect them.

Passing: the plan looks for BOTH sides — the enqueue/producer
call site and the worker that consumes it — and says how it
would link them (a shared queue or job name, an outbox table,
or a queue-relationship query).

Failing: the plan assumes a direct call chain, or searches only
for the producer and stops.
