# `no-mistakes tests impact`

Find impacted tests from explicit changed files.

```sh
no-mistakes tests impact src/api.mts --format json
```

Use this when an agent already knows the changed files and wants a structured
test set without parsing a git diff. Impact traversal is file-scoped today;
`file#symbol` inputs are accepted for compatibility but the symbol suffix does
not narrow the result.

Key options: `--root`, `--config`, `--tsconfig`, `--format`, and `--json`.

Node API: `testsImpact(options)`.
