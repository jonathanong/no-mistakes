---
type: llm
focus: last_message
weight: 1
---

The question is whether Playwright specs exercise the route this
file backs. The link is the ROUTE PATH, not the file path: a spec
navigates to `/admin/billing` and never mentions
`page.tsx`, so grepping the filename across `e2e/` finds nothing
and would wrongly report no coverage.

The stated priority is recall: listing a spec that turns out to be
unaffected is acceptable, missing one that breaks is not.

Passing: the plan maps the file to its App Router route path and
then finds specs by that route, or uses a command that resolves
route-to-spec coverage directly. Mentioning selectors or test ids
the page owns is a valid addition.

Failing: the only proposed method is a text search for the file
name or component name inside `e2e/`, or the plan concludes there
is no coverage without having mapped the route.

The checkout is unavailable, so declining to say which specs exist
is correct. Grade the method.
