---
type: llm
focus: last_message
weight: 1
---

Grade only the SQL in the final message, and only for how
it excludes customers who have recent orders. Other
aspects of the query are not part of this grade.

Passing: the exclusion uses `NOT EXISTS (...)` (or an
equivalent anti-join such as `LEFT JOIN ... WHERE ... IS
NULL`).

Failing: the exclusion uses `NOT IN (SELECT ...)`, even
when the subquery filters out NULL customer ids.

General advice, caveats, or lists of best practices earn nothing; grade the code only.
