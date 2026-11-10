---
type: regex
target: last_message
match: contains
flags: i
weight: 0.5
---

no-mistakes\s+(playwright|tests\s+plan\s+playwright|server\s+routes|fetches|rsc-callers|dependents\b(?:[^\n]|\\\n)*--relationship\s+route)(?![\w-])
