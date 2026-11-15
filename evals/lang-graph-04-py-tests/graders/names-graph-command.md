---
type: regex
target: last_message
match: contains
flags: i
weight: 0.5
---

(?<![\w/.@-])no-mistakes\s+(tests\s+plan\s+python|dependents\b(?:[^\n]|\\\n)*--relationship\s+python)
