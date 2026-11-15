---
type: regex
target: last_message
match: contains
flags: i
weight: 0.5
---

(?<![\w/.@-])no-mistakes\s+(playwright|tests\s+plan\s+playwright)(?![\w-])
