---
type: llm
focus: last_message
weight: 1
---

The plan aims for complete reference coverage before renaming, and
treats a missed reference as the failure mode that matters.

Passing: covers at least three of these categories explicitly — call
sites; imports and re-exports (including the package barrel); aliased or
renamed imports; type-only or type-position references; test files.

Failing: covers two or fewer, or proposes a single symbol-name search as
sufficient.

Explicitly: returning extra or over-broad candidates is NOT a defect and
must not reduce the score. Only missing a category counts against it.
