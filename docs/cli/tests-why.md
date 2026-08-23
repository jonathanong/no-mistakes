# `no-mistakes tests why`

Explain the dependency path from a changed file to a selected or skipped test.

```sh
no-mistakes tests why tests/users.test.mts --plan plan.json
no-mistakes tests why tests/users.test.mts --changed src/api.mts --format json
```

Use this when an agent needs to justify a targeted test set or investigate why
a test was missing.

Key options: `--root`, `--config`, `--tsconfig`, `--changed`, `--plan`, and
`--format text|json`.

Node API: `testsWhy(options)`.

For a resource edge, JSON steps include optional `detail` provenance with the
consumer file and sorted literal call sites. This is the same data preserved in
a plan's `via_details` field. Vitest setup steps likewise expose
`{ "type": "vitest-setup", "field": "setupFiles" | "globalSetup" }` in
`detail`.
