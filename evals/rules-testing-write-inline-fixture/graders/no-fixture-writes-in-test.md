---
type: llm
focus: last_message
weight: 1
---

Grade only the test code in the final message.

Passing: the test file itself never creates files or
directories. It reads manifests from pre-saved fixture
directories (the fixture files are shown as separate
blocks or referenced by a path under a fixtures
directory), and the missing-manifest case points at a
directory that has no manifest.json without creating one.

Failing: any test code calls `writeFile`, `mkdir`,
`mkdtemp`, `outputFile`, `cp`, or similar to build the
manifest or directory at test time, including in a
`beforeEach` or a helper defined in the test file.

General advice, caveats, or lists of best practices earn nothing; grade the code only.
