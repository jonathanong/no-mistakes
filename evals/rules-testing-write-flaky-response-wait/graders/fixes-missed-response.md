---
type: llm
focus: last_message
weight: 1
---

Grade only the replacement test code in the final message.

Passing: the code registers a wait for the POST to
/api/labels before the Save label click can send it.
It awaits both the click and the matching response and
checks that the response succeeded. It adds no retries.

Failing: the code can still attach the wait after the
click, adds any retry setting or rerun loop, replaces
the response wait with a delay, or drops the response
assertion. Advice without replacement code fails.

General advice, caveats, or lists of best practices earn nothing; grade the code only.
