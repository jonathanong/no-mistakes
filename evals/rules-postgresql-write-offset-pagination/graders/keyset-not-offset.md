---
type: llm
focus: last_message
weight: 1
---

Grade only the code in the final message.

Passing: the query pages with a keyset condition on the
sort columns, such as `(created_at, id) < ($2, $3)`, with
an `ORDER BY` in the same direction, a bound `LIMIT`, and
the caller passing the last row's values back as the
cursor.

Failing: the SQL contains `OFFSET` (or a computed
`page * size` skip), anywhere, even alongside `LIMIT`.

General advice, caveats, or lists of best practices earn nothing; grade the code only.
