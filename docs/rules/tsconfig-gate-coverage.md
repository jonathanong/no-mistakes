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

Workflow commands run only when their effective shell is GitHub Actions'
implicit shell or a static `bash`/`sh` form. The rule honors workflow and job
`defaults.run.shell` plus a step-level `shell` override; static shell templates
must invoke `bash` or `sh`, pass the script as `{0}`, and use only
execution-preserving flags: `-e`, `-u`, `-x`, and Bash's `-o pipefail`,
`--noprofile`, and `--norc`. This includes GitHub Actions' standard
`bash --noprofile --norc -eo pipefail {0}` and `sh -e {0}` templates.
Other shells (such as `python`, PowerShell, or `cmd`) and dynamic/custom shell
forms do not count; neither do non-executing modes such as `bash -n {0}`.

Literal YAML `if: false` and `continue-on-error: true` values, plus exact
constant expressions `${{ false }}` and `${{ true }}`, on a workflow job or
step do not count as CI registrations because they cannot enforce a typecheck.
Other expressions in either field remain unresolved and are not evaluated by
this static rule.

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
Shell bodies containing `exit`, `return`, `false`, or a failure-mode mutation
such as `set +e` are also rejected as a whole because the rule does not model
shell reachability or option state.

Informational and setup commands (`--showConfig`, `--help`/`-h`,
`--version`/`-v`, and `--init`) do not count, even when combined with
`--noEmit`, because they do not compile the project.

Findings use line 1 of the tsconfig, workflow, or configuration file. Use a
top-of-file `no-mistakes-disable-file tsconfig-gate-coverage` directive only
when an intentional exception cannot be represented with a reasoned
`allowProjects` entry.
