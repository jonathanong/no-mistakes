# `tsconfig-gate-coverage`

Requires every tracked `tsconfig.json` or `tsconfig.*.json` outside
`node_modules` to be registered in both a configured GitHub Actions workflow
and a local whole-project typecheck command.

```yaml
checks:
  commands:
    - name: typecheck-web
      command: [pnpm, --dir, web, exec, tsc, --noEmit]
      fileArgs: none
      always: true

rules:
  - rule: tsconfig-gate-coverage
    scope: repository
    options:
      allowProjects:
        web/tsconfig.dependency-cruiser.json: Used only by dependency-cruiser, not tsc.
```

The rule recognizes static `tsc --noEmit` commands in workflow `run:` steps
and `checks.commands`. It supports `--project <path>` / `--project=<path>`, a
default `tsconfig.json` relative to the effective working directory, sequential
`cd` commands, and `pnpm --dir <path> exec tsc`. Workflow working directories
may come from workflow/job `defaults.run.working-directory` or a step's
`working-directory`.

Counterexample: `packages/api/tsconfig.json` exists, but its `tsc --noEmit`
command appears only in a local command catalog. CI can therefore miss type
errors in that package.

Fix: add the matching static typecheck command to a configured workflow and a
`checks.commands` entry with `always: true` and `fileArgs: none`.
Auxiliary configs that intentionally are not compiler projects need a
non-empty reason in `options.allowProjects`; stale, blank, invalid, or
normalization-colliding entries fail the rule.

Dynamic shell expansion, command substitution, arbitrary wrapper scripts,
paths outside the repository, and other unresolved command forms do not count
as registrations. Express such a command statically if it is intended to
provide this gate.

Findings use line 1 of the tsconfig, workflow, or configuration file. Use a
top-of-file `no-mistakes-disable-file tsconfig-gate-coverage` directive only
when an intentional exception cannot be represented with a reasoned
`allowProjects` entry.
