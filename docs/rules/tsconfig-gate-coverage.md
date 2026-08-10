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
Step-based jobs need a non-empty, static `runs-on` string or label array.
Static local reusable-workflow jobs (`uses: ./.github/workflows/*.yml`) are
followed transitively and their step-based jobs are evaluated under the direct
caller's file triggers. Remote, dynamic, missing, non-callable, and cyclic
calls do not provide coverage. One complete, enforcing, acyclic caller path is
sufficient; partial coverage from separate caller paths is never combined.
The containing workflow must declare at least one file-triggered `push`,
`pull_request`, or `pull_request_target` event whose path filters allow every
visible TypeScript/JavaScript source selected by that project's
`files`/`include`/`exclude` settings. Projects with no known source files fall
back to the tracked tsconfig path. Input values and path coverage are evaluated
for each direct caller event independently, so coverage from different events
is never combined. An explicitly activity-filtered `pull_request` or
`pull_request_target` event must include `synchronize`, the event that runs when
source commits are added to an open pull request. Manual, scheduled, reusable,
empty, tag-only, and path-filtered-out workflows cannot provide a repository
typecheck gate on their own. A `workflow_call` workflow can provide one when
reached from an applicable caller. For
example, `paths: [app/tsconfig.json]` cannot cover `app/src/index.ts`; add
`app/**` or an
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
Jobs that declare `container` or `services` count only with a statically Linux
runner label. GitHub does not support those fields on Windows or macOS runners,
and an unknown custom runner label cannot prove the required Linux host.

Literal YAML `if: false` and `continue-on-error: true` values, plus exact
constant expressions `${{ false }}` and `${{ true }}`, on a workflow job or
step do not count as CI registrations because they cannot enforce a typecheck.
For reusable calls, the rule also resolves exact boolean input references and
negations/comparisons in call-job, callee-job, and step conditions. It also
short-circuits logical `&&` and `||` expressions when a known input operand
determines the result. Literal call inputs, declared defaults, omitted values,
and exact `${{ inputs.name }}` forwarding preserve boolean, string, and number
values through transitive calls. This lets the rule resolve exact string and
number equality/inequality comparisons as well as input truthiness. Expressions
whose result remains dynamic fail open as potentially runnable. Exact
`${{ matrix.name }}` bindings also preserve a scalar value when every generated
static matrix combination has the same value after `exclude` and `include`
expansion; non-uniform or dynamic matrices remain unresolved.
Condition expressions must also use contexts available at their location.
For example, job conditions cannot read `secrets`, while step conditions can
read `steps`, `runner`, and `env`. A malformed or unavailable context prevents
the workflow from providing coverage.

The rule evaluates the successful gate path: `success()`, `always()`, and
`!cancelled()` are runnable there, while `failure()` and `cancelled()` are not.
This prevents failure-handler or cancellation-only typechecks from satisfying
the required CI gate.

A job blocked by a statically skipped `needs` dependency, including a
transitive dependency, does not count. A job condition that contains a status
check such as `always()` or `!cancelled()` can explicitly continue after a
skipped need when the whole condition is statically true. A dependency with
`continue-on-error: true` is non-enforcing itself but is not treated as skipped
for downstream jobs.

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
shell reachability or option state. Negated pipelines and bodies with unquoted shell comments,
quoted command separators, or shell function/group braces, and local shell
invocations that enable a
non-executing mode such as `bash -n`, are rejected rather
than credited heuristically. A typecheck before another command in an `&&`
list is rejected when a later top-level command could mask a failed or skipped
typecheck. A final static `&&` list remains recognized.

Informational, setup, or config-bypassing commands (`--showConfig`,
`--help`/`-h`, `--version`/`-v`, `--init`, enabled `--noCheck`,
`--listFilesOnly`, and `--ignoreConfig`) do not count, even when combined with
`--noEmit`, because they do not fully typecheck the project. Explicit
`--noCheck false` and `--noCheck=false` forms remain typechecking modes.

Findings use line 1 of the tsconfig, workflow, or configuration file. Use a
top-of-file `no-mistakes-disable-file tsconfig-gate-coverage` directive only
when an intentional exception cannot be represented with a reasoned
`allowProjects` entry.
