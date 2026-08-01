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
and `checks.commands`. It supports `--project <path>` and `-p <path>`, a
default `tsconfig.json` relative to the effective
working directory, sequential `cd` commands, and
`pnpm --dir <path> exec tsc`. Workflow working directories
may come from workflow/job `defaults.run.working-directory` or a step's
`working-directory`.
Only step-based jobs with a non-empty, static `runs-on` string or label array
count; missing, dynamic, or reusable-workflow jobs do not.
The containing workflow must declare at least one file-triggerable `push`,
`pull_request`, or `pull_request_target` event whose path filters allow the
tracked tsconfig path. Manual, scheduled, reusable, empty, tag-only, and
path-filtered-out workflows cannot provide a repository typecheck gate. For
example, `paths: [docs/**]` cannot cover `app/tsconfig.json`; add `app/**` or an
unfiltered applicable event.

Workflow commands run only when their effective shell is GitHub Actions'
implicit shell or a static `bash`/`sh` form. The rule honors workflow and job
`defaults.run.shell` plus a step-level `shell` override; static shell templates
must invoke `bash` or `sh`, pass the script as `{0}`, and use only
execution-preserving flags: `-e`, `-u`, `-x`, and Bash's `-o pipefail`,
`--noprofile`, and `--norc`. This includes GitHub Actions' standard
`bash --noprofile --norc -eo pipefail {0}` and `sh -e {0}` templates.
Other shells (such as `python`, PowerShell, or `cmd`) and dynamic/custom shell
forms do not count; neither do non-executing modes such as `bash -n {0}`.
Implicit and built-in `bash`/`sh` shells propagate failures. Custom templates
must include `-e` or `-o errexit` to credit a typecheck before a later command;
without it, only a final `tsc` command counts.
An implicit shell does not count for statically Windows-labeled runners
(`windows-*` or a self-hosted `windows` label), because GitHub Actions defaults
those runners to PowerShell; specify a supported `bash` or `sh` shell instead.
A bare `self-hosted` label is also rejected with an implicit shell because its
operating system is not statically known; add a `linux`/`macos` label or an
explicit supported shell.

Literal YAML `if: false` and `continue-on-error: true` values, plus exact
constant expressions `${{ false }}` and `${{ true }}`, on a workflow job or
step do not count as CI registrations because they cannot enforce a typecheck.
Other expressions in either field remain unresolved and are not evaluated by
this static rule.

A project whose effective local `compilerOptions.noCheck` is `true` does not
count as typechecked, even when both commands are registered. Remove or disable
`noCheck`, or document an intentional non-typechecking project with
`allowProjects`. Local and installed-package `extends` chains are resolved
through the prepared source store; unresolved configs are left for `tsc` to
reject.

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
shell reachability or option state. Bodies with unquoted shell comments,
quoted command separators, or shell function/group braces, and local shell
invocations that enable a
non-executing mode such as `bash -n` or `set -o noexec`, are rejected rather
than credited heuristically.

Informational, setup, or config-bypassing commands (`--showConfig`,
`--help`/`-h`, `--version`/`-v`, `--init`, enabled `--noCheck`,
`--listFilesOnly`, and `--ignoreConfig`) do not count, even when combined with
`--noEmit`, because they do not fully typecheck the project. Explicit
`--noCheck false` and `--noCheck=false` forms remain typechecking modes.

Findings use line 1 of the tsconfig, workflow, or configuration file. Use a
top-of-file `no-mistakes-disable-file tsconfig-gate-coverage` directive only
when an intentional exception cannot be represented with a reasoned
`allowProjects` entry.
