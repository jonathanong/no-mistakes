---
type: llm
focus: last_message
weight: 1
---

The code that writes to this outbox and the worker that drains it
share no import edge, so following imports alone will never
connect them.

Passing: the plan identifies the consumer side by the queue or job
identity — queue name, job type, handler registration — rather
than by import traversal alone, and does not assume there is
exactly one consumer.

Failing: the plan treats this as an ordinary import-graph
question, or stops at the enqueue site and describes the producer
instead of the consumers.

The checkout is unavailable, so naming the consumer is not
expected. Grade whether the plan would find every consumer if
someone executed it.
