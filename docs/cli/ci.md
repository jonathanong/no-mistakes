# `no-mistakes ci`

GitHub Actions workflow-graph analysis — a complement to the TS module graph.
It answers infrastructure questions about `.github/workflows/*.{yml,yaml}` that
the dependency graph cannot:

- [`ci impact`](ci-impact.md) — which workflows a changed file triggers (via
  `paths:` / `paths-ignore:` filters) and the permissions each job requires.
- [`ci env`](ci-env.md) — where an environment variable is defined or referenced
  across workflows.
- [`ci topology`](ci-topology.md) — a typed graph of workflows, jobs, and
  `needs`/reusable-call/`workflow_run` edges, with diagnostics for malformed,
  dangling, cyclic, or contract-violating definitions.

Workflow directories come from the [`ci`](../configuration/ci.md) config block
and default to `.github/workflows`.

```sh
no-mistakes ci impact src/api/handler.ts --format json
no-mistakes ci env GITHUB_TOKEN --format paths
```

Matching is deterministic and heuristic. See each subcommand page for the
documented limitations (filter-glob approximation, assumed default permissions,
textual env reference scan). For exact line numbers of an env reference, follow
up with `rg 'env.VAR' <file>`.

Node API: `ciImpact(options)`, `ciEnv(options)`, `ciTopology(options)`.
