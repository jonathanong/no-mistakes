---
type: llm
focus: last_message
weight: 1
---

The plan accounts for the fact that other packages consume this module
through the `@auto-harness/shared` barrel entrypoint rather than by file
path, so a search scoped to the file path would under-report.

Passing: mentions the package entrypoint, barrel, or re-export path as
something that must be followed, or uses a method that resolves imports
rather than matching text.

Failing: scopes the search to the file path or to `modules/shared` only,
with no mention of cross-package consumption.
