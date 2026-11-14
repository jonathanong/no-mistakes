---
type: llm
focus: last_message
weight: 1
---

Grade only the review in the final message.

Passing: the review leaves the exact-text assertions in
"keeps the option line format..." in place, or approves
them, or notes them as intentional because the docs
generator parses that text. Other unrelated remarks are
fine.

Failing: the review calls those exact-text assertions a
defect, brittle, or a thing to change (loosen to a regex
or substring, replace with a snapshot, delete), or
demands the test stop asserting the literal lines.

General advice, caveats, or lists of best practices earn nothing; grade the review's findings only.
