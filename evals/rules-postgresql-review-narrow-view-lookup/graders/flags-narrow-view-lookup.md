---
type: llm
focus: last_message
weight: 1
---

Grade only the review in the final message.

Passing: the review flags joining `user_directory`
when the query only reads `handle` from the base
`users` table (pulling in the view's extra join to
`organizations` and/or correlated `team_memberships`
subquery unnecessarily), and recommends joining
`users` directly while keeping the view's
`deleted_at IS NULL` predicate.

Failing: the review leaves the `user_directory` join
in place, misses that the view carries extra
joins/subqueries for a single base-table column, or
recommends joining `users` directly without
preserving the `deleted_at IS NULL` predicate.

General advice, caveats, or lists of best practices earn nothing; grade the review's findings only.
