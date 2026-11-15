---
type: regex
target: last_message
match: contains
flags: i
weight: 0.5
---

(?<![\w/.@-])no-mistakes\s+(check|symbols|exports-of|dead-exports)(?![\w-])
