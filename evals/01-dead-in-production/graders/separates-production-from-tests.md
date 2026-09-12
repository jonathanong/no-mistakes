---
type: llm
focus: last_message
weight: 1
---

The plan establishes that usage inside test files does NOT make the
function live, and commits to separating production consumers from
test-only consumers before reaching any verdict on deleting it.

Passing responses do at least one of: state explicitly that test
references must be excluded or counted separately; describe a step that
filters or partitions consumers by production vs test.

Failing responses do any of: treat any reference to the symbol as proof
it is in use; conclude it is dead or safe to delete because the filename
says "legacy"; give a verdict with no plan to separate the two groups.
