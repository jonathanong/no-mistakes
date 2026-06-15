# `no-mistakes rsc-callers`

Find the server components and pages that transitively import a component
through React Server Component (RSC) boundaries.

```sh
no-mistakes rsc-callers app/ui/Button.tsx --root . --format json
```

Use this to see which server components and App Router pages render a component
without crossing a client boundary. Traversal walks the reverse import graph
from the component over runtime import edges. A `"use client"` file is a client
boundary: it is not reported (the query wants *server* callers) and the upward
RSC chain stops there, since anything rendered above a client boundary renders
the client subtree rather than the target directly. Files with `"use server"`
or no directive (App Router defaults to server components) are reported and
traversal continues through them.

Each caller is classified as a `page` (an App Router routing file such as
`page`, `route`, `layout`, `template`, `default`, `loading`, `error`, or
`not-found`) or a `component`, with its `environment` (`server`/`unknown`) and
import `depth` from the target. The page classification is a filename heuristic.

Key options: `--depth`, `--tsconfig`, `--config`, `--format`, and `--json`.

Output shape:

```json
{
  "component": "app/ui/Button.tsx",
  "callers": [
    { "file": "app/ui/Card.tsx", "kind": "component", "environment": "unknown", "depth": 1 },
    { "file": "app/dashboard/page.tsx", "kind": "page", "environment": "unknown", "depth": 2 }
  ]
}
```

A component imported nowhere yields an empty `callers` list.

## Limitation

Classification is by directive only. A file with no `"use client"`/`"use server"`
directive is treated as a server component (the App Router default), so
non-RSC importers without a directive — unit tests, stories, or helper modules
outside the App Router — are reported as `environment: unknown` callers. Scope
the result to the App Router (e.g. with `--root app`) or filter test/story files
out of the output when that matters.

Node API: `rscCallers(options)`.
