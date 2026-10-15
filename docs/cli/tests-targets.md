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
rendered with `--workspace`, including explicitly configured `.json` project
arrays; ordinary `vitest.config.*` sources use `--config`.

Key options: `--root`, `--config`, `--framework`, `--format`, and `--json`.

Node API: `testsTargets(options)`.

Cargo targets use the owning visible `Cargo.toml`, its `[package].name`, and
an enabled `--lib`, `--bin <name>`, or `--test <name>` target. Unit modules under
`src/**/tests/` are not integration targets. Custom target paths and disabled
auto-discovery are read from the manifest. Feature-gated targets are omitted
when their required features cannot be established by the planner.

Exact entrypoints and unambiguous target directories establish ownership;
shared library/binary source directories and unrelated integration helpers do
not. Such files retain `targets: []` and a `cargo-target-unresolved` warning.
The commands format rejects warnings instead of printing a partial runnable
plan. Make the owning manifest/entrypoint visible or select a known target
explicitly. This does not run Cargo metadata or infer a repository-wide suite.

Visible exact Cargo target entries outside their declaring package directory
retain that declaring package and selector. When several selected packages
declare the same entry, all enabled targets are returned deterministically.
Configured package scopes still apply to the declaring packages; external
entries do not make neighboring helpers or other packages eligible.
