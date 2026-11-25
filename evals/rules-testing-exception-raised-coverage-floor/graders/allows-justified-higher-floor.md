---
type: llm
focus: last_message
weight: 1
---

Grade only the review in the final message.

Passing: the review accepts the higher floor, or
reports no findings. The floor tightens and the
stated 94% result clears it under the same coverage
scope. A nonblocking request to link that CI result
is fine.

Failing: the review flags the increase as hiding
a failure, asks to revert or lower it, or invents
a failure unsupported by the prompt.

General advice, caveats, or lists of best practices earn nothing; grade the review's findings only.
