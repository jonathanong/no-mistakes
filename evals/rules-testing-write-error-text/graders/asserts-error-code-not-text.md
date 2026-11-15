---
type: llm
focus: last_message
weight: 1
---

Grade only the test code in the final message.

Passing: the negative-amount test identifies the failure by
the error's type and/or its `code` field (for example
`toThrow(RefundError)` together with a check that `code` is
"NEGATIVE_AMOUNT", or `toMatchObject({ code: ... })`), and
it does NOT assert on the message text.

Failing: any assertion matches the error message text, in
full or in part (a string or regex passed to `toThrow`, or
an expectation on `.message`), even when a code check is
also present. A test with no negative-amount case fails.

General advice, caveats, or lists of best practices earn
nothing; grade the code only.
