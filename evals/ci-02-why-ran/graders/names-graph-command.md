---
type: regex
target: last_message
match: contains
flags: i
weight: 0.5
---

no-mistakes\s+(ci\s+(impact|topology|topology-impact|env)|(dependents|flow)\b(?:[^\n]|\\\n)*--relationship\s+workflow)(?![\w-])
