# `no-mistakes check`

Run configured repository checks from `.no-mistakes.yml`.

```sh
no-mistakes check --root . --format json
```

Use this before finishing an agent edit when the repository has configured
rules. `check` runs React, queue, integration, filesystem, Playwright, unique
export, and codebase rules that are enabled in config.

Key options: `--root`, `--config`, `--tsconfig`, `--format`, and `--json`.
The root-global `--timings` and `--verbose-timings` flags work here and on every
other CLI leaf. Verbose mode implies timings, includes rule/graph/Playwright
sub-phases and work counts, and marks overlapping check-domain spans as
non-additive. See [Performance diagnostics](diagnostics.md).

Rules must be explicitly configured. See [no-mistakes rules](../rules/README.md)
and [configuration](../configuration/README.md).

If a configured check cannot run, `check` prints a warning to stderr, includes it
in structured output as `warnings`, and exits nonzero.

Node API: `check(options)`.
