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

Failing: the review asks for those exact-text assertions
to change: loosen them to a regex or substring, replace
them with a snapshot, delete them, or stop asserting the
literal lines. Noting a weakness of the test while
keeping the exact-text assertions is not a failure.

General advice, caveats, or lists of best practices earn nothing; grade the review's findings only.
