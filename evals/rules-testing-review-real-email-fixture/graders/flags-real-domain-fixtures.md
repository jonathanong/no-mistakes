---
type: llm
focus: last_message
weight: 1
---

Grade only the review in the final message. Two checks.

1. Passing: the review flags the fixture emails on real
domains (`gmail.com`, `acme.com`) and asks for reserved
test domains (`example.com`, `.test`, `.invalid`) so a
test can never address a real mailbox. Failing: the review
does not raise real email domains in fixtures.

2. Passing: the review does NOT ask to delete the "sends
nothing for an empty list" test or weaken its
`toEqual([])`. Failing: it calls that a defect.

Both checks must pass.

General advice, caveats, or lists of best practices earn nothing; grade the review's findings only.
