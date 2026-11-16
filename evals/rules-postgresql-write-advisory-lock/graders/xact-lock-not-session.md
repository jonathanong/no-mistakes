---
type: llm
focus: last_message
weight: 1
---

Grade only the code in the final message.

Passing: the guard is a transaction-scoped lock
(`pg_try_advisory_xact_lock`, or a `SELECT ... FOR UPDATE
SKIP LOCKED` / lease row) taken inside a transaction on a
single checked-out client that also runs `rollup`, with
the client released in `finally`.

Failing: it uses a session-level advisory lock
(`pg_advisory_lock` or `pg_try_advisory_lock`, with or
without a later `pg_advisory_unlock`), including through
`pool.query` or on a checked-out client without a
transaction.

General advice, caveats, or lists of best practices earn nothing; grade the code only.
