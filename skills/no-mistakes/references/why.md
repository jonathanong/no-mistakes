# Why no-mistakes exists

Coding agents miss structure that a text search cannot see. This skill exists
so they do not recreate an existing API, skip covering tests, ship an
untested App Router page, or treat an empty plan as complete.

`no-mistakes` parses the current checkout, builds one in-memory graph, and
returns a small structured result. It does not run the app, call a model, or
keep a database. Static literals produce edges; dynamic values are skipped.

## Three jobs

1. Scope the change (`dependents`, `symbols`, queues, server, workflow).
2. Select the tests (`tests plan`, `playwright related`).
3. Keep the graph honest (`check`, `playwright check`, ESLint rules).

## If you skip this

| Mistake | Use |
| --- | --- |
| Recreate an export grep found in one file | `unique-exports`, `dead-exports`, `symbols` |
| Run every Vitest or Playwright suite | `tests plan`, `playwright related` |
| Ship an App Router page with no test | `playwright check` |
| Treat an empty plan as complete | inspect `warnings` and `fallback_triggered` |
| Dynamic fetch URLs or test IDs | keep literals; `nextjs-static-fetch-url`, `playwright-literals` |
| Miss callers behind aliases | `dependents` (omit `--tsconfig` in monorepos) |
| Move a file and leave broken imports | `resolve-check` |
| Change a signature and miss callers | `symbols --mode signature-impact`, `call-sites` |
| Rename a queue job and miss the worker | `queues check`, `queues related` |
| Parse human CLI output | `--format json` |

Configured Python, Go, Rust, and other language graphs use
`dependents --relationship <lang>` first. Use `rg` for dynamic forms those
frontends skip.
