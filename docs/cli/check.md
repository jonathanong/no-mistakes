# `no-mistakes check`

Run configured repository checks from `.no-mistakes.yml`.

```sh
no-mistakes check --root . --format json
```

Use this before finishing an agent edit when the repository has configured
rules. `check` runs React, queue, integration, filesystem, Playwright, unique
export, and codebase rules that are enabled in config.

Key options: `--root`, `--config`, `--tsconfig`, `--format`, `--json`, and
`--timings`.

Rules must be explicitly configured. See [no-mistakes rules](../rules/README.md)
and [configuration](../configuration/README.md).

Node API: `check(options)`.
