---
type: llm
focus: last_message
weight: 1
---

Grade only the review in the final message. Two checks.

1. Passing: the review flags that the two scalar
subqueries look up the same `members` row twice per
output row via `t.assignee_id` and recommends
replacing them with a single join (for example a
`LEFT JOIN members a ON a.id = t.assignee_id AND
a.deleted_at IS NULL`) that projects both
`display_name` and `avatar_url`. Failing: the review
does not identify the duplicate lookup on
`t.assignee_id` or does not recommend collapsing it
into one join.

2. Passing: the review does NOT claim the `LEFT JOIN
members r ON r.id = t.reviewer_id` is a duplicate of
the `assignee_id` lookup or ask to merge
`reviewer_id` and `assignee_id` into one join.
Failing: it treats the distinct `reviewer_id` join as
a duplicate of `assignee_id`.

Both checks must pass.

General advice, caveats, or lists of best practices earn nothing; grade the review's findings only.
