---
type: llm
focus: last_message
weight: 1
---

The plan would find consumers that do not literally spell this file's
path — re-exports, barrel/index files, and imports written against a
package name rather than a relative path.

Passing: names at least one such indirect route and says how it would be
covered, or commits to a resolver/import-graph method that covers them
by construction.

Failing: the only proposed method is a text search for the symbol name
or the file path, with no acknowledgement that indirect imports exist.

Over-inclusiveness is not a defect. A plan that gathers extra candidate
consumers and then narrows them scores full marks.
