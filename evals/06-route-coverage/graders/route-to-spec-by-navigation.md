---
type: llm
focus: last_message
weight: 1
---

The plan determines Playwright coverage by what the specs actually
exercise — the route they navigate to — rather than by matching spec
filenames against the page's path.

Passing: describes locating specs that navigate to or request the route
(for example by inspecting navigation calls), or uses a tool that maps
routes to specs.

Failing: relies only on finding a spec whose filename resembles
"repositories", or assumes the absence of such a filename means no
coverage.
