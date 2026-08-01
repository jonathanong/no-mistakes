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
    - name: repository-policy
      always: true
      command: ["pnpm", "run", "repo-policy"]
      fileArgs: none
```

| Key | Default | Description |
| --- | --- | --- |
| `name` | `""` | Stable identifier used for reporting and dedupe. |
| `include` | `[]` | File globs (relative to root) that trigger the command. |
| `exclude` | `[]` | File globs that suppress the command. |
| `command` | `[]` | Command tokens, e.g. `[pnpm, exec, eslint]`. |
| `fileArgs` | `append` | `append` adds each matched file as a trailing argument; `none` runs the command once regardless of which files matched. |
| `always` | `false` | Run a whole-project command even with no changed files. It requires `fileArgs: none` and empty `include` and `exclude` lists. |

A command is emitted only when at least one changed file matches `include` and
is not excluded, unless it sets `always: true`. Always commands receive the
normalized changed-file list in their result metadata, including an empty list.
Use `fileArgs: none` for whole-project checks (typecheck, format-check) and
`fileArgs: append` for per-file linters.
Every `command` must start with a non-blank executable token.
