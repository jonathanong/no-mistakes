# `no-mistakes tests targets`

Resolve exact runner commands for known test files.

```sh
no-mistakes tests targets vitest web/app/users/users.test.ts --format json
no-mistakes tests targets playwright tests/e2e/login.spec.ts --format commands
```

`commands` prints shell-quoted test commands, deduped by configured execution
target. This is useful when another tool already selected test files and needs
the same project, package, and environment command construction as
`tests plan`. Vitest `vitest.workspace.*` and `vitest.projects.*` sources are
rendered with `--workspace`; ordinary `vitest.config.*` sources use `--config`.

Key options: `--root`, `--config`, `--framework`, `--format`, and `--json`.

Node API: `testsTargets(options)`.
