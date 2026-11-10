---
type: regex
target: last_message
match: contains
flags: i
weight: 0.5
---

no-mistakes\s+(queues\s+(related|edges|check)|dependents\b(?:[^\n]|\\\n)*--relationship\s+queue)(?![\w-])
