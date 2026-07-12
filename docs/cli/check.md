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

`--timings` reports the coarse, top-level phase breakdown (discover, parse_extract,
react, queues, rules, ...). For a deeper breakdown — which rule inside `rules`
dominates, or which `DepGraph` edge kind or Playwright analysis step is expensive —
add `--verbose-timings`. It prints `[timing] <label>: <ms>` lines to stderr (e.g.
`rules.forbidden_dependencies`, `graph.imports`, `playwright.test_file_analysis`) and
has no effect unless combined with `--timings`-style investigation; it exists so
diagnosing a performance regression doesn't require hand-editing `eprintln!` calls
into a special instrumented build. `--verbose-timings` is `check`-only today (the
same hot paths it instruments — rule dispatch, graph construction, Playwright
analysis — are reached from other commands too, but the flag hasn't been wired into
their arg structs yet).

Rules must be explicitly configured. See [no-mistakes rules](../rules/README.md)
and [configuration](../configuration/README.md).

If a configured check cannot run, `check` prints a warning to stderr, includes it
in structured output as `warnings`, and exits nonzero.

Node API: `check(options)`.
