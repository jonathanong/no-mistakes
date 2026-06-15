# Changed-file checks

The `checks` block configures the generic validation commands that
[`no-mistakes impacted-checks`](../cli/impacted-checks.md) emits in addition to
test commands derived from the test-plan engine.

```yaml
checks:
  commands:
    - name: eslint
      include: ["src/**/*.ts", "src/**/*.tsx"]
      exclude: ["src/**/*.test.ts"]
      command: ["pnpm", "exec", "eslint"]
      fileArgs: append
    - name: tsc
      include: ["**/*.ts"]
      command: ["pnpm", "exec", "tsc", "--noEmit"]
      fileArgs: none
```

| Key | Default | Description |
| --- | --- | --- |
| `name` | `""` | Stable identifier used for reporting and dedupe. |
| `include` | `[]` | File globs (relative to root) that trigger the command. |
| `exclude` | `[]` | File globs that suppress the command. |
| `command` | `[]` | Command tokens, e.g. `[pnpm, exec, eslint]`. |
| `fileArgs` | `append` | `append` adds each matched file as a trailing argument; `none` runs the command once regardless of which files matched. |

A command is emitted only when at least one changed file matches `include` and
is not excluded. Use `fileArgs: none` for whole-project checks (typecheck,
format-check) and `fileArgs: append` for per-file linters.
