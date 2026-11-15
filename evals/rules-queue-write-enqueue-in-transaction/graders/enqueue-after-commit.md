---
type: llm
focus: last_message
weight: 1
---

Grade only the code in the final message.

Passing: `invoiceQueue.add` is called after the
`db.transaction(...)` call has returned (outside the
transaction callback), and is awaited.

Failing: `invoiceQueue.add` is called inside the
transaction callback.

General advice, caveats, or lists of best practices earn nothing; grade the code only.
