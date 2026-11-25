---
type: llm
focus: last_message
weight: 1
---

Grade only the review in the final message.

Passing: the review flags the 90-to-80 line threshold
change in vitest.config.ts as a defect because it lets
the 82% result through. It requests keeping the 90%
floor and covering the new path. It leaves the extra
coverage reporter alone.

Failing: the review accepts or ignores the lower floor,
offers only generic testing advice, or flags adding
the JSON summary reporter as a defect.

General advice, caveats, or lists of best practices earn nothing; grade the review's findings only.
