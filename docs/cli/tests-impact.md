# `no-mistakes tests impact`

Find impacted tests from explicit `file#export` entrypoints.

```sh
no-mistakes tests impact src/api.mts#handler --format json
```

Use this when an agent already knows the changed public symbol and wants a
structured test set without parsing a git diff.

Key options: `--root`, `--config`, `--tsconfig`, `--format`, and `--json`.

Node API: `testsImpact(options)`.
