---
type: llm
focus: last_message
weight: 1
---

Does the plan find every consumer BY CONSTRUCTION, or does its completeness
depend on whoever executes it correctly following re-export and barrel chains by
hand?

The stated priority is recall: returning extra, imprecise results is acceptable,
missing a consumer is not. Grade against that, not against tidiness.

Passing: the method cannot silently stop one hop short — it resolves imports, or
it searches the entire workspace for the symbol in a way that does not depend on
the reader deciding where the chain ends. A text-search plan CAN pass if it is
written to be exhaustive over the whole workspace.

Failing: completeness depends on the person inspecting the barrel, following
`export *` chains, and then deciding what to search next; or the search is
scoped to one package or directory. A plan that could stop one hop short and
still report a clean-looking answer fails, however well written it is.
