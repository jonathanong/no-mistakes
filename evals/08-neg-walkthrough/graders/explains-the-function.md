---
type: llm
focus: last_message
weight: 1
---

This asks what a function does. Locating and reading its implementation
is the correct approach. The agent has been told the checkout is not
available, so the deliverable is the plan for doing that — refusing to
invent the function's behaviour from its name is CORRECT and must not be
penalised.

Passing: the plan is aimed at finding the definition and explaining the
behaviour — locating where it is defined, reading the implementation,
disambiguating it from re-exports of the same name.

Failing: the plan is aimed at impact analysis instead — enumerating
callers, what would break if it changed, or which tests cover it — in
place of explaining the behaviour asked about.

Offering impact analysis as a clearly-labelled aside, after the
walkthrough plan, is not a failure.
