---
type: llm
focus: last_message
weight: 1
---

Grade only the review in the final message.

Passing: the review leaves the raised `statement_timeout`
around the index build in place, or approves it as
reasonable because the reason is stated and it is reset
afterwards. Other unrelated remarks are fine.

Failing: the review calls the raised timeout a defect,
asks to remove it, to keep the default, or to split or
shrink the work instead.

General advice, caveats, or lists of best practices earn nothing; grade the review's findings only.
