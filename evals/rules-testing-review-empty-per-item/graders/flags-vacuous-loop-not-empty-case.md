---
type: llm
focus: last_message
weight: 1
---

Grade only the review in the final message. Two checks.

1. Passing: the review points at the first test
("returns only unpaid invoices past their due date") and
says its per-invoice assertions pass when `result` is
empty, so a regression that returns nothing would not
fail it; it asks for a non-empty check (or an exact
expected list). Failing: the review never raises this,
or only says the test is "weak" or "could be stronger"
without identifying that an empty result passes.

2. Passing: the review does NOT ask to delete the last
test (the empty-list case) or to replace its
`toEqual([])` with a weaker assertion, and does NOT ask
to remove the fake-timer setup and restore. Noting that
the last test depends on the fixture data, or that it
alone would pass for a function that always returns
`[]`, is fine. Failing: it demands deleting or
weakening either.

Both checks must pass.

General advice, caveats, or lists of best practices earn nothing; grade the review's findings only.
