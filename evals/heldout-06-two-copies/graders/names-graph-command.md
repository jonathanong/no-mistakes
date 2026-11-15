---
type: regex
target: last_message
match: contains
flags: i
weight: 0.5
---

(?<![\w/.@-])no-mistakes\s+(check|exports-of|symbols|dead-exports)(?![\w-])
